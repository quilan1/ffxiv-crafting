use std::{
    pin::Pin,
    sync::Arc,
    task::{Context, Poll, Waker},
};

use futures::{
    channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender},
    future::{BoxFuture, RemoteHandle},
    stream::BufferUnordered,
    Future, FutureExt, StreamExt,
};
use parking_lot::Mutex;

type FutureSender = UnboundedSender<BoxFuture<'static, ()>>;
type FutureReceiver = BufferUnordered<UnboundedReceiver<BoxFuture<'static, ()>>>;

#[derive(Clone)]
pub struct AsyncProcessor(Arc<AsyncProcessorInnerData>);

struct AsyncProcessorInnerData {
    tx: Mutex<FutureSender>,
    rx: Mutex<FutureReceiver>,
    waker: Mutex<Option<Waker>>,
    cur_id: Mutex<usize>,
    num_finished: Mutex<usize>,
}

pub struct AsyncProcessorHandle<T> {
    id: usize,
    handle: RemoteHandle<T>,
}

#[derive(Clone, Copy, PartialEq)]
pub enum AsyncProcessorStatus {
    Done,
    Active,
    Queued(usize),
}

////////////////////////////////////////////////////////////

// Consumer side of the API
impl AsyncProcessor {
    pub fn new(max_active: usize) -> Self {
        let (tx, rx) = unbounded();
        let rx = rx.buffer_unordered(max_active);
        Self(Arc::new(AsyncProcessorInnerData {
            tx: Mutex::new(tx),
            rx: Mutex::new(rx),
            waker: Mutex::new(None),
            cur_id: Mutex::new(0),
            num_finished: Mutex::new(0),
        }))
    }

    // Takes in future, queues it, and returns a future that you can poll for the result
    pub fn process_future<Fut>(&self, future: Fut) -> AsyncProcessorHandle<Fut::Output>
    where
        Fut: Future + Send + 'static,
        Fut::Output: Send,
    {
        let (future, remote) = future.remote_handle();

        let id = {
            let mut cur_id = self.0.cur_id.lock();
            let id = *cur_id;
            *cur_id += 1;
            id
        };

        let inner = self.0.clone();
        self.0
            .tx
            .lock()
            .unbounded_send(Box::pin(async move {
                future.await;
                if let Some(waker) = inner.waker.lock().as_ref() {
                    waker.wake_by_ref()
                }
            }))
            .unwrap();

        AsyncProcessorHandle { id, handle: remote }
    }

    pub fn disconnect(&self) {
        self.0.tx.lock().disconnect();
    }

    pub fn num_finished(&self) -> usize {
        *self.0.num_finished.lock()
    }
}

impl Future for AsyncProcessor {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.0.waker.lock().replace(cx.waker().clone());
        match self.0.rx.lock().poll_next_unpin(cx) {
            Poll::Ready(Some(_)) => {
                *self.0.num_finished.lock() += 1;
                Poll::Pending
            }
            Poll::Ready(None) => Poll::Ready(()),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<T> AsyncProcessorHandle<T> {
    pub fn id(&self) -> usize {
        self.id
    }
}

impl<T: 'static> Future for AsyncProcessorHandle<T> {
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.handle.poll_unpin(cx)
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
            let proc = AsyncProcessor::new(MAX_CONCURRENT);
            let count = AmValue::new(0);
            let ran_future = AmValue::new(false);

            async fn future(count: AmValue<usize>, ran_future: AmValue<bool>) {
                *ran_future.lock() = true;
                *count.lock() += 1;
                assert!(*count.lock() <= MAX_CONCURRENT);
                sleep(Duration::from_millis(10)).await;
                *count.lock() -= 1;
            }

            // The remote futures need to be stored, or else they're never run
            let _futures = (0..4)
                .map(|_| proc.process_future(future(count.clone(), ran_future.clone())))
                .collect::<Vec<_>>();

            proc.disconnect();
            proc.await;
            assert!(*ran_future.lock());
        });
    }
}
