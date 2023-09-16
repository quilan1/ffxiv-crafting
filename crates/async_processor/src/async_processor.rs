use std::{
    collections::{BTreeSet, HashMap},
    pin::Pin,
    task::{Context, Poll, Waker},
};

use futures::{Future, FutureExt};
use log::error;
use uuid::Uuid;

use crate::{AmoValue, ArwValue};

#[derive(Clone, Default)]
pub struct AsyncProcessor {
    active: ArwValue<Vec<IdFuture<()>>>,
    queue: ArwValue<Vec<IdFuture<()>>>,
    active_limited_ids: ArwValue<Vec<String>>,
    waker: AmoValue<Waker>,
    max_active: usize,
}

pub type SyncBoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send + Sync + 'static>>;

pub struct IdFuture<T>
where
    T: Send,
{
    pub id: String,
    pub future: SyncBoxFuture<T>,
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
    pub fn process_future<Fut>(&mut self, future: Fut) -> IdFuture<Fut::Output>
    where
        Fut: Future + Send + Sync + 'static,
        Fut::Output: Send,
    {
        let id = Uuid::new_v4().to_string();
        let (future, remote) = future.remote_handle();

        self.queue.write().push(IdFuture {
            id: id.clone(),
            future: Box::pin(future),
        });

        self.wake();
        IdFuture {
            id,
            future: Box::pin(remote),
        }
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
        let active_ids = self.active_limited_ids.read().clone();

        let queued_ids = self
            .queue
            .read()
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

    pub fn cancel(&self, ids: Vec<String>) {
        let ids = ids.into_iter().collect::<BTreeSet<_>>();
        self.active_limited_ids
            .write()
            .clone()
            .retain(|id| !ids.contains(id));
        self.queue.write().retain(|id| !ids.contains(&id.id));
        self.active.write().retain(|id| !ids.contains(&id.id));
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

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Set the waker, so it can be re-polled
        self.waker.replace(Some(cx.waker().clone()));

        // Keep the number of active futures limited to at most max_active
        let avail_slots = self
            .queue
            .read()
            .len()
            .min(self.max_active - self.active.read().len());

        // Move from queue to active
        self.active
            .write()
            .extend(self.queue.write().drain(..avail_slots));

        // Poll the active data
        self.active
            .write()
            .retain_mut(|fut| fut.future.as_mut().poll_unpin(cx).is_pending());

        // Update the active IDs for status
        let active_ids = self
            .active
            .read()
            .iter()
            .map(|id_future| id_future.id.clone())
            .collect();
        *self.active_limited_ids.write() = active_ids;

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
                .map(|_| proc.process_future(future(count.clone(), ran_future.clone())))
                .collect::<Vec<_>>();
            proc.process_all().await;
            assert!(*ran_future.lock());
        });
    }
}
