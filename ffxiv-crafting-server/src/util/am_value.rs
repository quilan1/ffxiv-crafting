#![allow(dead_code)]

use std::sync::Arc;

use parking_lot::{Mutex, MutexGuard};

#[derive(Default)]
pub struct AmValue<O> {
    data: Arc<Mutex<O>>,
}

pub type AmoValue<O> = AmValue<Option<O>>;

impl<O> AmValue<O> {
    pub fn new(v: O) -> Self {
        Self {
            data: Arc::new(Mutex::new(v)),
        }
    }

    pub fn lock(&self) -> MutexGuard<'_, O> {
        self.data.lock()
    }

    pub fn take(&self) -> O
    where
        O: Default,
    {
        let mut output = self.data.lock();
        std::mem::take(&mut output)
    }

    pub fn replace(&self, value: O) -> O {
        let mut output = self.data.lock();
        std::mem::replace(&mut output, value)
    }
}

impl<O> Clone for AmValue<O> {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
        }
    }
}
