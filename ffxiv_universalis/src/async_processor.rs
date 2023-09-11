use std::{
    pin::Pin,
    task::{Context, Poll, Waker},
};

use futures::{future::BoxFuture, Future, FutureExt};
use log::error;

use crate::{AmValue, AmoValue};

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
pub enum AsyncProcessType {
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
        process_type: AsyncProcessType,
    ) -> BoxFuture<'static, Fut::Output>
    where
        Fut: Future + Send + 'static,
        Fut::Output: Send,
    {
        let (future, remote) = future.remote_handle();
        match process_type {
            AsyncProcessType::Limited => self.queue.lock().limited.push(future.boxed()),
            AsyncProcessType::Unlimited => self.queue.lock().unlimited.push(future.boxed()),
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

    #[allow(dead_code, unused_must_use)]
    async fn process_all(&self) {
        while !self.is_empty() {
            futures::poll!(self.clone());
        }
    }

    fn is_empty(&self) -> bool {
        self.active.lock().is_empty() && self.queue.lock().is_empty()
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

impl AsyncProcessorData {
    fn is_empty(&self) -> bool {
        self.limited.is_empty() && self.unlimited.is_empty()
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

////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use tokio::{runtime::Builder, time::sleep};

    use super::*;

    fn block(f: impl Future<Output = ()>) {
        Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(f);
    }

    #[test]
    fn test_limited() {
        // Test that only MAX_CONCURRENT futures will run concurrently at a time, if stored as limited
        const MAX_CONCURRENT: usize = 2;

        block(async {
            let mut proc = AsyncProcessor::new(MAX_CONCURRENT);
            let count = AmValue::new(0);
            let ran_future = AmValue::new(false);

            async fn future(count: AmValue<usize>, ran_future: AmValue<bool>) {
                *ran_future.lock() = true;
                *count.lock() += 1;
                assert!(*count.lock() <= MAX_CONCURRENT);
                sleep(Duration::from_millis(10)).await;
                *count.lock() -= 1;
            }

            // The futures need to be stored, or else they're thrown away & never run
            let _futures = (0..4)
                .map(|_| {
                    proc.process_future(
                        future(count.clone(), ran_future.clone()),
                        AsyncProcessType::Limited,
                    )
                })
                .collect::<Vec<_>>();
            proc.process_all().await;
            assert!(*ran_future.lock());
        });
    }

    #[test]
    fn test_unlimited() {
        // Test that more than MAX_CONCURRENT futures will run concurrently at a time, if stored as unlimited
        const MAX_CONCURRENT: usize = 2;

        block(async {
            let mut proc = AsyncProcessor::new(MAX_CONCURRENT);
            let count = AmValue::new(0);
            let ran_future = AmValue::new(false);
            let was_above_max = AmValue::new(false);

            async fn future(
                count: AmValue<usize>,
                is_above_max: AmValue<bool>,
                ran_future: AmValue<bool>,
            ) {
                *ran_future.lock() = true;
                *count.lock() += 1;
                *is_above_max.lock() |= *count.lock() > MAX_CONCURRENT;
                sleep(Duration::from_millis(10)).await;
                *count.lock() -= 1;
            }

            // The futures need to be stored, or else they're thrown away & never run
            let _futures = (0..4)
                .map(|_| {
                    proc.process_future(
                        future(count.clone(), was_above_max.clone(), ran_future.clone()),
                        AsyncProcessType::Unlimited,
                    )
                })
                .collect::<Vec<_>>();
            proc.process_all().await;
            assert!(*was_above_max.lock());
            assert!(*ran_future.lock());
        });
    }
}
