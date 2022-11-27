use std::{
    collections::VecDeque,
    pin::Pin,
    sync::Arc,
    task::{Poll, Waker},
    time::Instant, fs::File, io::{BufWriter, Write},
};

use anyhow::{bail, Result};
use futures::{
    future::{join_all, BoxFuture, Shared},
    Future,
};
use parking_lot::Mutex;

use crate::util::AsyncCounter;

const MAX_QUEUE_SIZE: usize = 8;

#[derive(Clone)]
pub struct AsyncProcessor<'a, O> {
    data: Arc<Mutex<AsyncProcessorData<'a, O>>>,
}

pub type SharedFuture<'a, O> = Shared<BoxFuture<'a, O>>;

struct NotifyFuture<'a, O> {
    future: SharedFuture<'a, O>,
    counter: AsyncCounter,
}

struct AsyncProcessorData<'a, O> {
    active: Vec<NotifyFuture<'a, O>>,
    queue: VecDeque<NotifyFuture<'a, O>>,
    waker: Option<Waker>,
}

trait Inner {
    type Type;
    fn with_inner<T, F: FnMut(&mut Self::Type) -> T>(&self, func: F) -> T;
    fn try_into_inner(self) -> Result<Self::Type>;
    fn into_inner(self) -> Self::Type
    where
        Self: Sized,
    {
        self.try_into_inner().unwrap()
    }
}

pub trait Notify {
    fn notify(&self);
}

////////////////////////////////////////////////////////////

// Consumer side of the API
impl<'a, O> AsyncProcessor<'a, O> {
    pub fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(AsyncProcessorData {
                active: Vec::new(),
                queue: VecDeque::new(),
                waker: None,
            }))
        }
    }

    pub async fn process(&self, values: Vec<SharedFuture<'a, O>>) -> Vec<O>
    where
        O: Clone,
    {
        if values.len() == 0 {
            return Vec::new();
        }

        let counter = AsyncCounter::new(values.len() as u32);
        self.queue_futures(values.clone(), counter.clone());

        counter.await;
        join_all(values).await
    }

    fn queue_futures(&self, futures: Vec<SharedFuture<'a, O>>, counter: AsyncCounter)
    where
        O: Clone,
    {
        let mut data = self.data.lock();

        // Move the futures into the queue
        for future in futures {
            let notified_future = NotifyFuture {
                future,
                counter: counter.clone(),
            };
            data.queue.push_back(notified_future);
        }

        // Poll processor again
        data.waker.as_ref().unwrap().wake_by_ref();
    }
}

// impl<'a, O> AsyncProcessorData<'a, O> {
//     #[allow(dead_code, unused_variables)]
// }

impl<'a, O> Future for AsyncProcessor<'a, O>
where
    O: Clone,
{
    type Output = O;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let mut data = self.data.lock();

        // self.log(format!(
        //     "== [Poll] Start (active={}, queue={}) ==",
        //     data.active.len(),
        //     data.queue.len()
        // ));

        // Keep the queue limited to MAX_QUEUE_SIZE
        while data.active.len() < MAX_QUEUE_SIZE {
            match data.queue.pop_front() {
                Some(future) => data.active.push(future),
                None => break,
            }
        }

        // Filter out any completed futures
        let mut notified_futures = data.active.drain(..).collect::<Vec<_>>();
        for mut notified_future in notified_futures.drain(..) {
            match Pin::new(&mut notified_future.future).poll(cx) {
                Poll::Pending => data.active.push(notified_future),
                Poll::Ready(_) => notified_future.counter.notify(),
            }
        }

        // Set the waker, so it can be re-polled
        data.waker = Some(cx.waker().clone());
        // self.log(format!(
        //     "== [Poll] Done (active={}, queue={}) ==",
        //     data.active.len(),
        //     data.queue.len()
        // ));

        // This future never ends
        Poll::Pending
    }
}

impl<'a, O> Inner for AsyncProcessor<'a, O> {
    type Type = AsyncProcessorData<'a, O>;

    fn with_inner<T, F: FnMut(&mut Self::Type) -> T>(&self, mut func: F) -> T {
        let mut data = self.data.lock();
        func(&mut data)
    }

    fn try_into_inner(self) -> Result<Self::Type> {
        match Arc::try_unwrap(self.data) {
            Err(_) => bail!("Couldn't unwrap rc"),
            Ok(v) => Ok(v.into_inner()),
        }
    }
}
