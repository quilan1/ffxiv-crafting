use std::{
    collections::VecDeque,
    pin::Pin,
    sync::Arc,
    task::{Poll, Waker},
};

use anyhow::{bail, Result};
use futures::{
    future::{join_all, BoxFuture, Shared},
    Future,
};
use parking_lot::Mutex;

#[allow(unused_imports)]
use log::info;

use crate::util::AsyncCounter;

#[derive(Clone)]
pub struct AsyncProcessor<'a, O> {
    data: Arc<Mutex<AsyncProcessorData<'a, O>>>,
}

pub type SharedFuture<'a, O> = Shared<BoxFuture<'a, O>>;

struct NotifyFuture<'a, O> {
    future: SharedFuture<'a, O>,
    counter: AsyncCounter,
}

#[allow(dead_code)]
struct AsyncProcessorData<'a, O> {
    active: Vec<NotifyFuture<'a, O>>,
    queue: VecDeque<NotifyFuture<'a, O>>,
    waker: Option<Waker>,
    name: String,
    max_queue: usize,
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
    pub fn new<S: Into<String>>(name: S, max_queue: usize) -> Self {
        Self {
            data: Arc::new(Mutex::new(AsyncProcessorData {
                active: Vec::new(),
                queue: VecDeque::new(),
                waker: None,
                name: name.into(),
                max_queue,
            })),
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
        if let Some(waker) = data.waker.as_ref() {
            waker.wake_by_ref();
        }
    }
}

impl<'a, O> Future for AsyncProcessor<'a, O>
where
    O: Clone,
{
    type Output = O;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let mut data = self.data.lock();

        // info!(
        //     "[Poll {}] Start (active={}, queue={})",
        //     data.name,
        //     data.active.len(),
        //     data.queue.len()
        // );

        // Keep the queue limited to max_queue
        while data.active.len() < data.max_queue {
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
        // info!(
        //     "[Poll {}] Done (active={}, queue={})",
        //     data.name,
        //     data.active.len(),
        //     data.queue.len()
        // );

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
