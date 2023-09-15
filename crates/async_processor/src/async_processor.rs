use std::{
    collections::HashMap,
    pin::Pin,
    task::{Context, Poll, Waker},
};

use futures::{Future, FutureExt};
use log::error;
use uuid::Uuid;

use crate::{AmoValue, ArwValue};

#[derive(Clone, Default)]
pub struct AsyncProcessor {
    active: ArwValue<AsyncProcessorData>,
    queue: ArwValue<AsyncProcessorData>,
    waker: AmoValue<Waker>,
    max_active: usize,
}

#[derive(Default)]
pub struct AsyncProcessorData {
    limited: Vec<IdFuture<()>>,
    unlimited: Vec<IdFuture<()>>,
}

pub type SyncBoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send + Sync + 'static>>;

pub struct IdFuture<T>
where
    T: Send,
{
    pub id: String,
    pub future: SyncBoxFuture<T>,
}

#[derive(Clone, Copy)]
pub enum AsyncProcessType {
    Limited,
    Unlimited,
}

#[derive(Clone)]
pub enum AsyncProcessStatus {
    Queued(usize),
    Active,
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
    ) -> IdFuture<Fut::Output>
    where
        Fut: Future + Send + Sync + 'static,
        Fut::Output: Send,
    {
        let id = Uuid::new_v4().to_string();
        let (future, remote) = future.remote_handle();

        let id_future = IdFuture {
            id: id.clone(),
            future: Box::pin(future),
        };

        match process_type {
            AsyncProcessType::Limited => self.queue.write().limited.push(id_future),
            AsyncProcessType::Unlimited => self.queue.write().unlimited.push(id_future),
        };

        self.wake();
        IdFuture {
            id,
            future: Box::pin(remote),
        }
    }

    // Move stuff from the queues to the active lists
    fn move_from_queue_to_active(&mut self) {
        let mut active = self.active.write();
        let mut queue = self.queue.write();

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

    pub fn status(&self) -> HashMap<String, AsyncProcessStatus> {
        let active_ids = self
            .active
            .read()
            .limited
            .iter()
            .map(|id_future| id_future.id.clone())
            .collect::<Vec<_>>();

        let queued_ids = self
            .queue
            .read()
            .limited
            .iter()
            .enumerate()
            .map(|(index, id_future)| (index, id_future.id.clone()))
            .collect::<Vec<_>>();

        let mut id_map = HashMap::new();
        id_map.extend(
            active_ids
                .into_iter()
                .map(|id| (id, AsyncProcessStatus::Active)),
        );
        id_map.extend(
            queued_ids
                .into_iter()
                .map(|(index, id)| (id, AsyncProcessStatus::Queued(index))),
        );
        id_map
    }

    // Polls the active futures, without holding the lock during the polling
    fn poll_active(&mut self, cx: &mut Context<'_>) -> Poll<()> {
        use std::mem::swap;

        // There are two mildly difficult aspects to this, that need to be handled
        // 1) The active queue needs to maintain a list of the correct ids
        // 2) The polled futures must not be called while under lock
        //
        // Therefore, I've created a replacement AsyncProcessorData object, to have
        // the futures swapped into. This way, the actual 'active' object will have dummy
        // futures, and the 'data' object will have the real futures.
        // Once 'data' has been polled, then we swap back 'data' with the 'active' object.

        fn swap_out_id_futures(from: &mut Vec<IdFuture<()>>, to: &mut Vec<IdFuture<()>>) {
            for id_future in from {
                // Create a dummy future, which doesn't do anything
                let mut temp_id_future = IdFuture {
                    id: id_future.id.clone(),
                    future: Box::pin(async {}),
                };
                swap(&mut temp_id_future, id_future);
                to.push(temp_id_future);
            }
        }

        let mut data = AsyncProcessorData::default();
        {
            let mut active = self.active.write();
            swap_out_id_futures(&mut active.limited, &mut data.limited);
            swap_out_id_futures(&mut active.unlimited, &mut data.unlimited);
        }

        let result = data.poll_unpin(cx);

        std::mem::swap(&mut data, &mut *self.active.write());

        result
    }

    #[cfg(test)]
    #[allow(unused_must_use)]
    async fn process_all(&self) {
        while !self.is_empty() {
            futures::poll!(self.clone());
        }
    }

    #[cfg(test)]
    fn is_empty(&self) -> bool {
        self.active.read().is_empty() && self.queue.read().is_empty()
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

        // Poll the active data
        self.poll_active(cx)
    }
}

impl AsyncProcessorData {
    #[cfg(test)]
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
            .retain_mut(|fut| fut.future.as_mut().poll_unpin(cx).is_pending());
        self.unlimited
            .retain_mut(|fut| fut.future.as_mut().poll_unpin(cx).is_pending());

        Poll::Pending
    }
}

////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use tokio::{runtime::Builder, time::sleep};

    use crate::AmValue;

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
