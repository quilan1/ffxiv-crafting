use std::{
    collections::VecDeque,
    mem::replace,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll, Waker},
};

use futures::{future::BoxFuture, Future, FutureExt};
use log::error;
use parking_lot::Mutex;

#[allow(unused_imports)]
use log::info;

use crate::util::AsyncCounter;

use super::AmoValue;

#[derive(Clone)]
pub struct AsyncProcessor {
    data: Arc<Mutex<AsyncProcessorData>>,
}

pub struct FutureOutput<O>(AsyncCounter, Vec<AmoValue<O>>);

struct NotifyFuture<Fut, O> {
    future: Fut,
    output: AmoValue<O>,
    counter: AsyncCounter,
}

struct AsyncProcessorData {
    active: Vec<BoxFuture<'static, ()>>,
    queue: VecDeque<BoxFuture<'static, ()>>,
    waker: Option<Waker>,
    max_active: usize,
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

    // Takes a set of futures, and returns future-output data, for storing the results
    pub fn process<Fut>(&self, futures: Vec<Fut>) -> FutureOutput<Fut::Output>
    where
        Fut: Future + Unpin + Send + 'static,
        Fut::Output: Send,
    {
        // Get the counter ready, to wait for when the futures are finished
        let counter = AsyncCounter::new(futures.len() as u32);

        // Create some output storage for the results
        let outputs = futures.iter().map(|_| AmoValue::new()).collect::<Vec<_>>();

        // Package the futures & outputs together
        let futures = futures.into_iter().zip(outputs.clone()).collect();

        // Queue the modified futures
        self.queue_futures(futures, counter.clone());
        FutureOutput(counter, outputs)
    }

    // Adds the futures to the internal queue system of the AsyncProcessor
    fn queue_futures<Fut>(&self, futures: Vec<(Fut, AmoValue<Fut::Output>)>, counter: AsyncCounter)
    where
        Fut: Future + Unpin + Send + 'static,
        Fut::Output: Send,
    {
        let mut data = self.data.lock();
        data.queue_futures(futures, counter);
    }
}

impl AsyncProcessorData {
    fn queue_futures<Fut>(
        &mut self,
        futures: Vec<(Fut, AmoValue<Fut::Output>)>,
        counter: AsyncCounter,
    ) where
        Fut: Future + Send + 'static,
        Fut::Output: Send,
        NotifyFuture<Fut, Fut::Output>: Future<Output = ()>,
    {
        self.queue
            .extend(futures.into_iter().map(|(future, output)| {
                NotifyFuture {
                    future,
                    output,
                    counter: counter.clone(),
                }
                .boxed()
            }));

        // Wake up the processor, so it can take a look at the queue & move them into active polling
        if let Some(waker) = self.waker.as_ref() {
            waker.wake_by_ref();
        } else {
            error!("AsyncProcessorData waker does not exist?! This usually means the processor is not currently\
                   'await'ing somewhere. Might cause orphan futures.");
        }
    }
}

// Abstract out the notify future polling
impl<Fut> Future for NotifyFuture<Fut, Fut::Output>
where
    Fut: Future + Unpin,
{
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.future.poll_unpin(cx) {
            Poll::Ready(output) => {
                self.output.replace(output);
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

// For stored-value futures (instead of copying future outputs all around), this allows us to have a () future,
// with an output value
impl<O> Future for FutureOutput<O> {
    type Output = Vec<O>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.0.poll_unpin(cx) {
            Poll::Ready(_) => {
                let v = replace(&mut self.1, Vec::new());
                Poll::Ready(v.into_iter().map(|value| value.take().unwrap()).collect())
            }
            Poll::Pending => Poll::Pending,
        }
    }
}
