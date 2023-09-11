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

////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lock() {
        let i = AmValue::new(3);
        assert_eq!(*i.lock(), 3);
        *i.lock() = 4;
        assert_eq!(*i.lock(), 4);
    }

    #[test]
    fn test_replace() {
        let i = AmValue::new(3);
        assert_eq!(*i.lock(), 3);
        i.replace(4);
        assert_eq!(*i.lock(), 4);
    }

    #[test]
    fn test_clone() {
        let i = AmValue::new(3);
        let _i = i.clone();

        struct NoClone {}
        let i = AmoValue::new(Some(NoClone {}));
        let _i = i.clone();
    }
}
