use std::fmt::Display;

use crate::util::AmValue;

#[derive(Clone)]
pub struct UniversalisStatus {
    data: AmValue<UniversalisStatusValue>,
}

#[derive(Clone)]
enum UniversalisStatusValue {
    Queued,
    Remaining(usize),
    Processing,
    Finished,
}

// Some utility functions for the status, to prevent accidentally deadlocking via the mutex (oops)
impl UniversalisStatus {
    pub fn new() -> Self {
        Self::default()
    }

    #[allow(dead_code)]
    pub fn is_finished(&self) -> bool {
        let data = self.data.lock();
        data.is_finished()
    }

    pub fn try_set_count(&self, count: usize) {
        let mut data = self.data.lock();
        if let UniversalisStatusValue::Queued = *data {
            *data = UniversalisStatusValue::Remaining(count);
        }
    }

    pub fn dec_count(&self) {
        let mut data = self.data.lock();
        if let UniversalisStatusValue::Remaining(count) = *data {
            *data = UniversalisStatusValue::Remaining(count - 1);
        }
    }

    pub fn set_processing(&self) {
        let mut data = self.data.lock();
        *data = UniversalisStatusValue::Processing;
    }

    pub fn set_finished(&self) {
        let mut data = self.data.lock();
        *data = UniversalisStatusValue::Finished;
    }
}

impl Default for UniversalisStatus {
    fn default() -> Self {
        Self {
            data: AmValue::new(UniversalisStatusValue::Queued),
        }
    }
}

impl Display for UniversalisStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let data = self.data.lock();
        write!(f, "{data}")
    }
}

#[allow(dead_code)]
impl UniversalisStatusValue {
    fn is_finished(&self) -> bool {
        matches!(*self, UniversalisStatusValue::Finished)
    }
}

// String value for the status
impl Display for UniversalisStatusValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            UniversalisStatusValue::Queued => write!(f, "Queued..."),
            UniversalisStatusValue::Remaining(count) => write!(f, "Remaining: {count}"),
            UniversalisStatusValue::Processing => write!(f, "Processing..."),
            UniversalisStatusValue::Finished => write!(f, "Finished"),
        }
    }
}
