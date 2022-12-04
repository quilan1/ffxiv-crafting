use std::{
    collections::VecDeque,
    pin::Pin,
    sync::Arc,
    task::{Poll, Waker},
};

use futures::{
    future::{join_all, BoxFuture, Shared},
    Future, FutureExt,
};
use log::error;
use parking_lot::Mutex;

#[allow(unused_imports)]
use log::info;

use crate::util::AsyncCounter;

#[derive(Clone)]
pub struct AsyncProcessor {
    data: Arc<Mutex<AsyncProcessorData>>,
}

pub type SharedFuture<O> = Shared<BoxFuture<'static, O>>;

struct NotifyFuture<O> {
    future: SharedFuture<O>,
    counter: AsyncCounter,
}

struct AsyncProcessorData {
    active: Vec<BoxFuture<'static, ()>>,
    queue: VecDeque<BoxFuture<'static, ()>>,
    waker: Option<Waker>,
    max_active: usize,
}

pub trait Notify {
    fn notify(&self);
}

////////////////////////////////////////////////////////////

// Consumer side of the API
impl AsyncProcessor {
    pub fn new(max_active: usize) -> Self {
        Self {
            data: Arc::new(Mutex::new(AsyncProcessorData {
                active: Vec::new(),
                queue: VecDeque::new(),
                waker: None,
                max_active,
            })),
        }
    }

    // Takes a vector of shared futures, and sends the through the count-limited queue and yields the results
    pub async fn process<O>(&self, values: Vec<SharedFuture<O>>) -> Vec<O>
    where
        O: Clone + Send + Sync + 'static,
    {
        if values.len() == 0 {
            return Vec::new();
        }

        // The counter acts as a way of waiting for all futures to finish, before we await them
        let counter = AsyncCounter::new(values.len() as u32);
        self.queue_futures(values.clone(), counter.clone());

        // At this point, the values can't be awaited directly, or they'll all run at the same time
        // Thus, we wait for the counter to count down to zero, then it will finish its future
        counter.await;

        // Now that the counter's zero, we know all futures have been resolved, so we can safely
        // await their values
        join_all(values).await
    }

    // Adds the futures to the internal queue system of the AsyncProcessor
    fn queue_futures<O>(&self, futures: Vec<SharedFuture<O>>, counter: AsyncCounter)
    where
        O: Clone + Send + Sync + 'static,
    {
        let mut data = self.data.lock();

        // Move the futures into the queue
        for future in futures {
            data.queue.push_back(
                NotifyFuture {
                    future,
                    counter: counter.clone(),
                }
                .boxed(),
            );
        }

        // Wake up the processor, so it can take a look at the queue & move them into active polling
        if let Some(waker) = data.waker.as_ref() {
            waker.wake_by_ref();
        } else {
            error!("AsyncProcessor waker does not exist?! This usually means the processor is not currently\
                   'await'ing somewhere. Might cause a zombie future.");
        }
    }
}

// Abstract out the notify future polling
impl<O: Clone> Future for NotifyFuture<O> {
    type Output = ();

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Self::Output> {
        match Pin::new(&mut self.future).poll(cx) {
            Poll::Ready(_) => {
                self.counter.notify();
                Poll::Ready(())
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

impl Future for AsyncProcessor {
    type Output = ();

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let mut data = self.data.lock();

        // info!(
        //     "[Poll] Start (active={}, queue={})",
        //     data.active.len(),
        //     data.queue.len()
        // );

        // Keep the queue limited to max_queue
        while data.active.len() < data.max_active {
            match data.queue.pop_front() {
                Some(future) => data.active.push(future),
                None => break,
            }
        }

        // Filter out any completed futures
        let mut notified_futures = data.active.drain(..).collect::<Vec<_>>();
        for mut notified_future in notified_futures.drain(..) {
            if notified_future.as_mut().poll(cx) == Poll::Pending {
                data.active.push(notified_future);
            }
        }

        // Set the waker, so it can be re-polled
        data.waker = Some(cx.waker().clone());

        // info!(
        //     "[Poll] Done (active={}, queue={})",
        //     data.active.len(),
        //     data.queue.len()
        // );

        // This future never ends
        Poll::Pending
    }
}
