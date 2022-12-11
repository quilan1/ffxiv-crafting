#![allow(dead_code)]

use std::sync::Arc;

use parking_lot::Mutex;

pub struct AmoValue<O> {
    data: Arc<Mutex<Option<O>>>,
}

impl<O> AmoValue<O> {
    pub fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(None)),
        }
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

impl<O> Clone for AmoValue<O> {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
        }
    }
}
