use std::{
    collections::VecDeque,
    pin::Pin,
    sync::Arc,
    task::{Poll, Waker},
};

use anyhow::{bail, Result};
use futures::{
    future::{join_all, BoxFuture, Shared},
    Future, FutureExt,
};
use parking_lot::Mutex;

#[allow(unused_imports)]
use log::info;

use crate::util::AsyncCounter;

#[derive(Clone)]
pub struct AsyncProcessor<'a> {
    data: Arc<Mutex<AsyncProcessorData<'a>>>,
}

pub type SharedFuture<'a, O> = Shared<BoxFuture<'a, O>>;

struct NotifyFuture<'a, O> {
    future: SharedFuture<'a, O>,
    counter: AsyncCounter,
}

#[allow(dead_code)]
struct AsyncProcessorData<'a> {
    active: Vec<BoxFuture<'a, ()>>,
    queue: VecDeque<BoxFuture<'a, ()>>,
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
impl<'a> AsyncProcessor<'a> {
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

    pub async fn process<O>(&self, values: Vec<SharedFuture<'a, O>>) -> Vec<O>
    where
        O: Clone + Send + Sync + 'a,
    {
        if values.len() == 0 {
            return Vec::new();
        }

        let counter = AsyncCounter::new(values.len() as u32);
        self.queue_futures(values.clone(), counter.clone());

        counter.await;
        join_all(values).await
    }

    fn queue_futures<O>(&self, futures: Vec<SharedFuture<'a, O>>, counter: AsyncCounter)
    where
        O: Clone + Send + Sync + 'a,
    {
        let mut data = self.data.lock();

        // Move the futures into the queue
        for future in futures {
            let notified_future = NotifyFuture {
                future,
                counter: counter.clone(),
            }.boxed();
            data.queue.push_back(notified_future);
        }

        // Poll processor again
        if let Some(waker) = data.waker.as_ref() {
            waker.wake_by_ref();
        }
    }
}

impl<'a, O: Clone> Future for NotifyFuture<'a, O> {
    type Output = ();

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Self::Output> {
        match Pin::new(&mut self.future).poll(cx) {
            Poll::Ready(_) => {
                self.counter.notify();
                Poll::Ready(())
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<'a> Future for AsyncProcessor<'a> {
    type Output = ();

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
            if Pin::new(&mut notified_future).poll(cx) == Poll::Pending {
                data.active.push(notified_future);
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

impl<'a> Inner for AsyncProcessor<'a> {
    type Type = AsyncProcessorData<'a>;

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
