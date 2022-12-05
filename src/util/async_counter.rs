use std::{
    pin::Pin,
    sync::Arc,
    task::{Context, Poll, Waker},
};

use futures::Future;
use parking_lot::Mutex;

use super::async_processor::Notify;

#[derive(Clone)]
pub struct AsyncCounter {
    count: Arc<Mutex<u32>>,
    waker: Arc<Mutex<Option<Waker>>>,
}

impl AsyncCounter {
    pub fn new(count: u32) -> Self {
        Self {
            count: Arc::new(Mutex::new(count)),
            waker: Arc::new(Mutex::new(None)),
        }
    }

    pub fn count(&self) -> u32 {
        *self.count.lock()
    }
}

impl Notify for AsyncCounter {
    fn notify(&self) {
        *self.count.lock() -= 1;
        if let Some(waker) = self.waker.lock().as_ref() {
            waker.wake_by_ref();
        }
    }
}

impl Future for AsyncCounter {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.waker.lock().replace(cx.waker().clone());
        match *self.count.lock() {
            0 => Poll::Ready(()),
            _ => Poll::Pending,
        }
    }
}
