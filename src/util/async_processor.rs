use std::{
    mem::take,
    pin::Pin,
    task::{Context, Poll, Waker},
};

use futures::{future::BoxFuture, Future, FutureExt};
use log::error;

use crate::util::AsyncCounter;

use super::{AmValue, AmoValue};

#[derive(Clone, Default)]
pub struct AsyncProcessor {
    active: AmValue<AsyncProcessorData>,
    queue: AmValue<AsyncProcessorData>,
    waker: AmoValue<Waker>,
    max_active: usize,
}

#[derive(Default)]
struct AsyncProcessorData {
    limited: Vec<BoxFuture<'static, ()>>,
    unlimited: Vec<BoxFuture<'static, ()>>,
}

struct NotifyFuture<Fut, O> {
    future: Fut,
    output: AmoValue<O>,
    counter: AsyncCounter,
}

pub trait ProcessFutures<I, O> {
    fn process_limited(&mut self, futures: I) -> FutureOutput<O>;
    fn process_unlimited(&mut self, futures: I) -> FutureOutput<O>;
}

pub struct FutureOutput<O>(AsyncCounter, O);
pub type FutureOutputOne<O> = FutureOutput<AmoValue<O>>;
pub type FutureOutputVec<O> = FutureOutput<Vec<AmoValue<O>>>;

////////////////////////////////////////////////////////////

// Consumer side of the API
impl AsyncProcessor {
    pub fn new(max_active: usize) -> Self {
        Self {
            max_active,
            ..Default::default()
        }
    }

    // Add a future, with the counter & output to either the unlimited or limited queue
    fn queue_future<Fut>(
        &mut self,
        future: Fut,
        output: &AmoValue<Fut::Output>,
        counter: &AsyncCounter,
        is_limited: bool,
    ) where
        Fut: Future + Send + 'static,
        Fut::Output: Send,
        NotifyFuture<Fut, Fut::Output>: Future<Output = ()>,
    {
        let mut queue = self.queue.lock();
        let queue = if is_limited {
            &mut queue.limited
        } else {
            &mut queue.unlimited
        };
        queue.push(NotifyFuture::new(future, output, counter).boxed());
    }

    // Move stuff from the queues to the active lists
    fn move_from_queue_to_active(&mut self) {
        let mut active = self.active.lock();
        let mut queue = self.queue.lock();

        // Keep the number of active futures limited to at most max_active
        let avail_slots = queue
            .limited
            .len()
            .min(self.max_active - active.limited.len());
        let moved_futures = queue.limited.drain(..avail_slots);
        active.limited.extend(moved_futures);

        // Move from the unlimited queue to active
        let moved_futures = queue.unlimited.drain(..);
        active.unlimited.extend(moved_futures);
    }

    // Wake up the processor, so it can take a look at the queue & move them into active polling
    fn wake(&self) {
        if let Some(waker) = self.waker.lock().as_ref() {
            waker.wake_by_ref();
        } else {
            error!("AsyncProcessorData waker does not exist?! This usually means the processor is not currently\
                   'await'ing somewhere. Might cause orphan futures.");
        }
    }

    // Create a counter, output, then queue & package the future
    fn process_one<Fut>(&mut self, future: Fut, is_limited: bool) -> FutureOutputOne<Fut::Output>
    where
        Fut: Future + Unpin + Send + 'static,
        Fut::Output: Send,
    {
        let counter = AsyncCounter::new(1);
        let output = AmoValue::new();
        self.queue_future(future, &output, &counter, is_limited);
        self.wake();

        FutureOutput(counter, output)
    }

    // Create a counter, outputs, then queue & package the futures
    fn process_many<Fut>(
        &mut self,
        futures: Vec<Fut>,
        is_limited: bool,
    ) -> FutureOutputVec<Fut::Output>
    where
        Fut: Future + Unpin + Send + 'static,
        Fut::Output: Send,
    {
        // Get the counter ready, to wait for when the futures are finished
        let counter = AsyncCounter::new(futures.len() as u32);

        // Create some output storage for the results
        let outputs = futures.iter().map(|_| AmoValue::new()).collect::<Vec<_>>();

        // Queue the futures
        for (future, output) in futures.into_iter().zip(outputs.clone()) {
            self.queue_future(future, &output, &counter, is_limited);
        }
        self.wake();

        FutureOutput(counter, outputs)
    }
}

impl<Fut, O> NotifyFuture<Fut, O> {
    fn new(future: Fut, output: &AmoValue<O>, counter: &AsyncCounter) -> Self {
        Self {
            future,
            output: output.clone(),
            counter: counter.clone(),
        }
    }
}

////////////////////////////////////////////////////////////

// Simply a pass-through for the data information
impl Future for AsyncProcessor {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Set the waker, so it can be re-polled
        self.waker.replace(cx.waker().clone());

        // Move from queue
        self.move_from_queue_to_active();

        // Poll!
        self.active.lock().poll_unpin(cx)
    }
}

// Polls the active lists for the processor, and retains only those that are unfinished
impl Future for AsyncProcessorData {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Keep only the unfinished futures
        self.limited
            .retain_mut(|fut| fut.poll_unpin(cx).is_pending());
        self.unlimited
            .retain_mut(|fut| fut.poll_unpin(cx).is_pending());

        Poll::Pending
    }
}

// Abstract out the notify future polling
impl<Fut> Future for NotifyFuture<Fut, Fut::Output>
where
    Fut: Future + Unpin,
{
    type Output = ();

    // If it's done, save the output and notify the counter, so it can finish
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

// These allow us to have copyable outputs without having to require clone() for the futures
impl<O> Future for FutureOutputVec<O> {
    type Output = Vec<O>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.0.poll_unpin(cx) {
            Poll::Ready(_) => {
                let v = take(&mut self.1);
                Poll::Ready(v.into_iter().map(|value| value.take().unwrap()).collect())
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

// These allow us to have a copyable output without having to require clone() for the future
impl<O> Future for FutureOutputOne<O> {
    type Output = O;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.0.poll_unpin(cx) {
            Poll::Ready(_) => {
                let v = take(&mut self.1);
                Poll::Ready(v.take().unwrap())
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

////////////////////////////////////////////////////////////

// Takes a vec of futures, and returns future-output data
impl<Fut> ProcessFutures<Vec<Fut>, Vec<AmoValue<Fut::Output>>> for AsyncProcessor
where
    Fut: Future + Unpin + Send + 'static,
    Fut::Output: Send,
{
    fn process_limited(&mut self, futures: Vec<Fut>) -> FutureOutputVec<Fut::Output> {
        self.process_many(futures, true)
    }

    fn process_unlimited(&mut self, futures: Vec<Fut>) -> FutureOutputVec<Fut::Output> {
        self.process_many(futures, false)
    }
}

// Takes a future, and returns future-output data
impl<Fut> ProcessFutures<Fut, AmoValue<Fut::Output>> for AsyncProcessor
where
    Fut: Future + Unpin + Send + 'static,
    Fut::Output: Send,
{
    fn process_limited(&mut self, future: Fut) -> FutureOutputOne<Fut::Output> {
        self.process_one(future, true)
    }

    fn process_unlimited(&mut self, future: Fut) -> FutureOutputOne<Fut::Output> {
        self.process_one(future, false)
    }
}
