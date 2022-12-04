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

#[allow(dead_code)]
struct AsyncProcessorData {
    active: Vec<BoxFuture<'static, ()>>,
    queue: VecDeque<BoxFuture<'static, ()>>,
    waker: Option<Waker>,
    name: String,
    max_queue: usize,
}

pub trait Notify {
    fn notify(&self);
}

////////////////////////////////////////////////////////////

// Consumer side of the API
impl AsyncProcessor {
    pub fn new<S: Into<String>>(name: S, max_queue: usize) -> Self {
        Self {
            data: Arc::new(Mutex::new(AsyncProcessorData {
                active: Vec::new(),
                queue: VecDeque::new(),
                waker: None,
                name: name.into(),
                max_queue,
            })),
        }
    }

    pub async fn process<O>(&self, values: Vec<SharedFuture<O>>) -> Vec<O>
    where
        O: Clone + Send + Sync + 'static,
    {
        if values.len() == 0 {
            return Vec::new();
        }

        let counter = AsyncCounter::new(values.len() as u32);
        self.queue_futures(values.clone(), counter.clone());

        counter.await;
        join_all(values).await
    }

    fn queue_futures<O>(&self, futures: Vec<SharedFuture<O>>, counter: AsyncCounter)
    where
        O: Clone + Send + Sync + 'static,
    {
        let mut data = self.data.lock();

        // Move the futures into the queue
        for future in futures {
            let notified_future = NotifyFuture {
                future,
                counter: counter.clone(),
            }
            .boxed();
            data.queue.push_back(notified_future);
        }

        // Poll processor again
        if let Some(waker) = data.waker.as_ref() {
            waker.wake_by_ref();
        }
    }
}

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
        //     "[Poll {}] Start (active={}, queue={})",
        //     data.name,
        //     data.active.len(),
        //     data.queue.len()
        // );

        // Keep the queue limited to max_queue
        while data.active.len() < data.max_queue {
            match data.queue.pop_front() {
                Some(future) => data.active.push(future),
                None => break,
            }
        }

        // Filter out any completed futures
        let mut notified_futures = data.active.drain(..).collect::<Vec<_>>();
        for mut notified_future in notified_futures.drain(..) {
            if Pin::new(&mut notified_future).poll(cx) == Poll::Pending {
                data.active.push(notified_future);
            }
        }

        // Set the waker, so it can be re-polled
        data.waker = Some(cx.waker().clone());
        // info!(
        //     "[Poll {}] Done (active={}, queue={})",
        //     data.name,
        //     data.active.len(),
        //     data.queue.len()
        // );

        // This future never ends
        Poll::Pending
    }
}
