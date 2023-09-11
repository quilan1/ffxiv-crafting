use std::fmt::Display;

use crate::AmValue;

#[derive(Clone)]
pub struct UniversalisStatus {
    data: AmValue<UniversalisStatusValue>,
}

enum UniversalisStatusValue {
    Queued,
    Remaining(usize),
    Finished,
}

// Some utility functions for the status, to prevent accidentally deadlocking via the mutex (oops)
impl UniversalisStatus {
    pub(crate) fn new() -> Self {
        Self {
            data: AmValue::new(UniversalisStatusValue::Queued),
        }
    }

    pub(crate) fn try_set_count(&self, count: usize) {
        let mut data = self.data.lock();
        if let UniversalisStatusValue::Queued = *data {
            *data = UniversalisStatusValue::Remaining(count);
        }
    }

    pub(crate) fn dec_count(&self) {
        let mut data = self.data.lock();
        if let UniversalisStatusValue::Remaining(count) = *data {
            *data = UniversalisStatusValue::Remaining(count - 1);
        }
    }

    pub(crate) fn set_finished(&self) {
        let mut data = self.data.lock();
        *data = UniversalisStatusValue::Finished;
    }
}

impl Display for UniversalisStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let data = self.data.lock();
        write!(f, "{data}")
    }
}

// String value for the status
impl Display for UniversalisStatusValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            UniversalisStatusValue::Queued => write!(f, "Queued..."),
            UniversalisStatusValue::Remaining(count) => write!(f, "Remaining: {count}"),
            UniversalisStatusValue::Finished => write!(f, "Finished"),
        }
    }
}
