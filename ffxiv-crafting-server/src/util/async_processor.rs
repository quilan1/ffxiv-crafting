use std::{
    pin::Pin,
    task::{Context, Poll, Waker},
};

use futures::{future::BoxFuture, Future, FutureExt};
use log::error;

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

#[derive(Clone, Copy)]
pub enum ProcessType {
    Limited,
    Unlimited,
}

////////////////////////////////////////////////////////////

// Consumer side of the API
impl AsyncProcessor {
    pub fn new(max_active: usize) -> Self {
        Self {
            max_active,
            ..Default::default()
        }
    }

    // Takes in future, queues it, and returns a future that you can poll for the result
    pub fn process_future<Fut>(
        &mut self,
        future: Fut,
        process_type: ProcessType,
    ) -> BoxFuture<'static, Fut::Output>
    where
        Fut: Future + Send + 'static,
        Fut::Output: Send,
    {
        let (future, remote) = future.remote_handle();
        match process_type {
            ProcessType::Limited => self.queue.lock().limited.push(future.boxed()),
            ProcessType::Unlimited => self.queue.lock().unlimited.push(future.boxed()),
        };
        self.wake();
        remote.boxed()
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
}

////////////////////////////////////////////////////////////

// Moves items from the queue into the active lists, and then polls the active items
impl Future for AsyncProcessor {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Set the waker, so it can be re-polled
        self.waker.replace(Some(cx.waker().clone()));

        // Move from queue
        self.move_from_queue_to_active();

        // Poll!
        self.active.lock().poll_unpin(cx)
    }
}

// Polls the active lists and retains only those futures that are unfinished
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
