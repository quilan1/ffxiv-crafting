use std::{fmt::Debug, sync::Arc};

use anyhow::Result;
use parking_lot::Mutex;
use tokio::sync::broadcast::{channel, Receiver, Sender};

////////////////////////////////////////////////////////////

/// A struct that will contain a shared value and a reciever
/// for signals sent from a broadcast channel. This structure
/// maybe cloned, to access the recievers & values elsewhere.
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
        *self.value.lock() = value.clone();
        self.tx.send(value)?;
        Ok(())
    }
}

impl<T: Clone + Send + Sync + 'static> MReceiver<T> {
    /// Retrieves a cloned copy of the value inside the MReceiver.
    pub fn get(&self) -> T {
        (*self.value.lock()).clone()
    }

    /// Retrieves a copy of the inner Receiver.
    pub fn receiver(&self) -> Receiver<T> {
        self.rx.resubscribe()
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
