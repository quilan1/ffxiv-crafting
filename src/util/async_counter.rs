#![allow(dead_code)]

use std::{
    pin::Pin,
    task::{Context, Poll, Waker},
};

use futures::Future;

use super::{AmValue, AmoValue};

#[derive(Clone)]
pub struct AsyncCounter {
    count: AmValue<u32>,
    waker: AmoValue<Waker>,
}

impl AsyncCounter {
    pub fn new(count: u32) -> Self {
        Self {
            count: AmValue::with_value(count),
            waker: AmoValue::new(),
        }
    }

    pub fn count(&self) -> u32 {
        *self.count.lock()
    }

    pub fn notify(&self) {
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
