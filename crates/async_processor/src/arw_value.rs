use std::sync::Arc;

use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};

#[derive(Default)]
pub struct ArwValue<O> {
    data: Arc<RwLock<O>>,
}

pub type ArwoValue<O> = ArwValue<Option<O>>;

impl<O> ArwValue<O> {
    pub fn new(v: O) -> Self {
        Self {
            data: Arc::new(RwLock::new(v)),
        }
    }

    pub fn read(&self) -> RwLockReadGuard<'_, O> {
        self.data.read()
    }

    pub fn write(&self) -> RwLockWriteGuard<'_, O> {
        self.data.write()
    }

    pub fn replace(&self, value: O) -> O {
        let mut output = self.data.write();
        std::mem::replace(&mut output, value)
    }
}

impl<O> Clone for ArwValue<O> {
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
        let i = ArwValue::new(3);
        assert_eq!(*i.read(), 3);
        assert_eq!(*i.write(), 3);
        *i.write() = 4;
        assert_eq!(*i.read(), 4);
        assert_eq!(*i.write(), 4);
    }

    #[test]
    fn test_replace() {
        let i = ArwValue::new(3);
        assert_eq!(*i.read(), 3);
        assert_eq!(*i.write(), 3);
        i.replace(4);
        assert_eq!(*i.read(), 4);
        assert_eq!(*i.write(), 4);
    }

    #[test]
    fn test_clone() {
        let i = ArwValue::new(3);
        let _i = i.clone();

        struct NoClone {}
        let i = ArwValue::new(Some(NoClone {}));
        let _i = i.clone();
    }
}
