use std::{
    collections::VecDeque,
    mem::take,
    pin::Pin,
    task::{Context, Poll, Waker},
};

use futures::{future::BoxFuture, Future, FutureExt};
use log::error;

use crate::util::AsyncCounter;

use super::{AmValue, AmoValue};

pub struct FutureOutput<O>(AsyncCounter, O);
pub type FutureOutputOne<O> = FutureOutput<AmoValue<O>>;
pub type FutureOutputVec<O> = FutureOutput<Vec<AmoValue<O>>>;

type SFuture = BoxFuture<'static, ()>;

struct NotifyFuture<Fut, O> {
    future: Fut,
    output: AmoValue<O>,
    counter: AsyncCounter,
}

#[derive(Clone, Default)]
pub struct AsyncProcessor {
    active: AmValue<AsyncProcessorActive>,
    queue: AmValue<AsyncProcessorQueue>,
    waker: AmoValue<Waker>,
    max_active: usize,
}

#[derive(Default)]
struct AsyncProcessorActive {
    active: Vec<SFuture>,
    active_unlimited: Vec<SFuture>,
}

#[derive(Default)]
struct AsyncProcessorQueue {
    queue: VecDeque<SFuture>,
    queue_unlimited: Vec<SFuture>,
}

pub trait ProcessFutures<I, O> {
    fn process(&mut self, futures: I, unlimited: bool) -> FutureOutput<O>;
}

////////////////////////////////////////////////////////////

// Consumer side of the API
impl AsyncProcessor {
    pub fn new(max_active: usize) -> Self {
        Self {
            active: AmValue::with_value(AsyncProcessorActive::default()),
            queue: AmValue::with_value(AsyncProcessorQueue::default()),
            waker: AmoValue::new(),
            max_active,
        }
    }

    fn queue_future<Fut>(
        &mut self,
        future: Fut,
        output: &AmoValue<Fut::Output>,
        counter: &AsyncCounter,
        unlimited: bool,
    ) where
        Fut: Future + Send + 'static,
        Fut::Output: Send,
        NotifyFuture<Fut, Fut::Output>: Future<Output = ()>,
    {
        let notify_future = NotifyFuture::new(future, output, counter).boxed();

        let mut queue = self.queue.lock();
        if unlimited {
            queue.queue_unlimited.push(notify_future);
        } else {
            queue.queue.push_back(notify_future);
        }
    }

    fn move_from_queue_to_active(&mut self) {
        let mut active = self.active.lock();
        let mut queue = self.queue.lock();

        // Keep the number of active futures limited to at most max_active
        let avail_slots = queue.queue.len().min(self.max_active - active.active.len());
        let moved_futures = queue.queue.drain(..avail_slots).collect::<Vec<_>>();
        active.active.extend(moved_futures);

        // Move from the unlimited queue to active
        let moved_futures = queue.queue_unlimited.drain(..).collect::<Vec<_>>();
        active.active_unlimited.extend(moved_futures);
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

// The main polling routine for the AsyncProcessor. Consumes any available futures from the queue, and moves them into
// the active list. All futures in the active list are then polled, and anything that returns ready is removed.
impl Future for AsyncProcessorActive {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Keep only the unfinished futures
        self.active
            .retain_mut(|fut| fut.poll_unpin(cx).is_pending());
        self.active_unlimited
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
    fn process(
        &mut self,
        futures: Vec<Fut>,
        unlimited: bool,
    ) -> FutureOutput<Vec<AmoValue<Fut::Output>>> {
        // Get the counter ready, to wait for when the futures are finished
        let counter = AsyncCounter::new(futures.len() as u32);

        // Create some output storage for the results
        let outputs = futures.iter().map(|_| AmoValue::new()).collect::<Vec<_>>();

        // Queue the futures
        for (future, output) in futures.into_iter().zip(outputs.clone()) {
            self.queue_future(future, &output, &counter, unlimited);
        }
        self.wake();

        FutureOutput(counter, outputs)
    }
}

// Takes a future, and returns future-output data
impl<Fut> ProcessFutures<Fut, AmoValue<Fut::Output>> for AsyncProcessor
where
    Fut: Future + Unpin + Send + 'static,
    Fut::Output: Send,
{
    fn process(&mut self, future: Fut, unlimited: bool) -> FutureOutput<AmoValue<Fut::Output>> {
        let counter = AsyncCounter::new(1);
        let output = AmoValue::new();
        self.queue_future(future, &output, &counter, unlimited);
        self.wake();

        FutureOutput(counter, output)
    }
}
