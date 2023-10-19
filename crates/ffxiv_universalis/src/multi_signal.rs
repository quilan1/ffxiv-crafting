use std::{
    fmt::Debug,
    pin::{pin, Pin},
    sync::{Arc, Mutex},
    task::{Context, Poll},
};

use anyhow::Result;
use futures::{Future, FutureExt};
use tokio::sync::broadcast::{channel, error::RecvError, Receiver, Sender};

////////////////////////////////////////////////////////////

pub struct MReceiver<T: Clone + Send + Sync + 'static> {
    value: Arc<Mutex<T>>,
    rx: Receiver<T>,
}

pub struct MSender<T: Clone + Send + Sync + 'static> {
    value: Arc<Mutex<T>>,
    tx: Sender<T>,
}

////////////////////////////////////////////////////////////

pub fn multi_signal<T: Clone + Send + Sync + 'static>(
    value: T,
    size: usize,
) -> (MSender<T>, MReceiver<T>) {
    let value = Arc::new(Mutex::new(value));
    let (tx, rx) = channel::<T>(size);
    let sender = MSender {
        value: value.clone(),
        tx,
    };
    let receiver = MReceiver { value, rx };
    (sender, receiver)
}

impl<T: Clone + Send + Sync + 'static> MSender<T> {
    pub fn set(&mut self, value: T) -> Result<()>
    where
        T: Debug,
    {
        *self.value.lock().unwrap() = value.clone();
        self.tx.send(value)?;
        Ok(())
    }
}

impl<T: Clone + Send + Sync + 'static> MReceiver<T> {
    pub fn get(&self) -> T {
        (*self.value.lock().unwrap()).clone()
    }
}

impl<T: Clone + Send + Sync + 'static> Future for MReceiver<T> {
    type Output = Result<T, RecvError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut fut = pin!(self.rx.recv());
        fut.poll_unpin(cx)
    }
}

impl<T: Clone + Send + Sync + 'static> Clone for MReceiver<T> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            rx: self.rx.resubscribe(),
        }
    }
}
