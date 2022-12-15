#![allow(dead_code)]

use std::sync::Arc;

use parking_lot::{Mutex, MutexGuard};

pub struct AmoValue<O> {
    data: Arc<Mutex<Option<O>>>,
}

pub struct AmValue<O> {
    data: Arc<Mutex<O>>,
}

impl<O> AmoValue<O> {
    pub fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(None)),
        }
    }

    pub fn lock(&self) -> MutexGuard<'_, Option<O>> {
        self.data.lock()
    }

    pub fn with_value(v: O) -> Self {
        Self {
            data: Arc::new(Mutex::new(Some(v))),
        }
    }

    pub fn take(&self) -> Option<O> {
        let mut output = self.data.lock();
        std::mem::replace(&mut *output, None)
    }

    pub fn replace(&self, value: O) -> Option<O> {
        self.data.lock().replace(value)
    }
}

impl<O> AmValue<O> {
    pub fn lock(&self) -> MutexGuard<'_, O> {
        self.data.lock()
    }

    pub fn with_value(v: O) -> Self {
        Self {
            data: Arc::new(Mutex::new(v)),
        }
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

impl<O> Clone for AmoValue<O> {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
        }
    }
}

impl<O> Clone for AmValue<O> {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
        }
    }
}

impl<O> Default for AmoValue<O> {
    fn default() -> Self {
        Self::new()
    }
}

impl<O> Default for AmValue<O>
where
    O: Default,
{
    fn default() -> Self {
        Self::with_value(O::default())
    }
}
