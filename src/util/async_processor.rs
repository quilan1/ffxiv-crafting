use std::{
    collections::VecDeque,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll, Waker},
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
        // By awaiting the lazy process, we ensure all futures are finished
        self.process_lazy(values.clone()).await;
        join_all(values).await
    }

    pub fn process_lazy<O>(&self, values: Vec<SharedFuture<O>>) -> AsyncCounter
    where
        O: Clone + Send + Sync + 'static,
    {
        // The counter is the primative that will allow us to know when the futures have all been executed
        let counter = AsyncCounter::new(values.len() as u32);
        self.queue_futures(values, counter.clone());
        counter
    }

    // Adds the futures to the internal queue system of the AsyncProcessor
    fn queue_futures<O>(&self, futures: Vec<SharedFuture<O>>, counter: AsyncCounter)
    where
        O: Clone + Send + Sync + 'static,
    {
        let mut data = self.data.lock();

        // Move the futures into the queue
        data.queue.extend(futures.into_iter().map(|future| {
            NotifyFuture {
                future,
                counter: counter.clone(),
            }
            .boxed()
        }));

        // Wake up the processor, so it can take a look at the queue & move them into active polling
        if let Some(waker) = data.waker.as_ref() {
            waker.wake_by_ref();
        } else {
            error!("AsyncProcessor waker does not exist?! This usually means the processor is not currently\
                   'await'ing somewhere. Might cause orphan futures.");
        }
    }
}

// Abstract out the notify future polling
impl<O> Future for NotifyFuture<O>
where
    SharedFuture<O>: Future,
{
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.future.poll_unpin(cx) {
            Poll::Ready(_) => {
                self.counter.notify();
                Poll::Ready(())
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

// Simply a pass-through for the data information
impl Future for AsyncProcessor {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.data.lock().poll_unpin(cx)
    }
}

// The main polling routine for the AsyncProcessor. Consumes any available futures from the queue, and moves them into
// the active list. All futures in the active list are then polled, and anything that returns ready is removed.
impl Future for AsyncProcessorData {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Keep the number of active futures limited to at most max_active
        let avail_slots = self.queue.len().min(self.max_active - self.active.len());
        let moved_futures = self.queue.drain(..avail_slots).collect::<Vec<_>>();
        self.active.extend(moved_futures);

        // Keep only the unfinished futures
        self.active
            .retain_mut(|fut| fut.poll_unpin(cx).is_pending());

        // Set the waker, so it can be re-polled
        self.waker.replace(cx.waker().clone());

        Poll::Pending
    }
}
