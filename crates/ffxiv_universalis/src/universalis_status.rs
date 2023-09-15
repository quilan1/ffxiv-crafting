use async_processor::{AmValue, AsyncProcessStatus, AsyncProcessor};

#[derive(Clone)]
pub struct UniversalisStatus {
    data: AmValue<UniversalisStatusValue>,
}

#[derive(Clone)]
pub enum UniversalisStatusValue {
    Queued,
    Processing(Vec<String>),
    Cleanup,
    Finished,
}

// Some utility functions for the status, to prevent accidentally deadlocking via the mutex (oops)
impl UniversalisStatus {
    pub(crate) fn new() -> Self {
        Self {
            data: AmValue::new(UniversalisStatusValue::Queued),
        }
    }

    pub(crate) fn set_value(&self, value: UniversalisStatusValue) {
        *self.data.lock() = value;
    }

    pub fn value(&self) -> UniversalisStatusValue {
        (*self.data.lock()).clone()
    }

    pub fn text(&self, processor: &AsyncProcessor) -> String {
        let ids = match self.value() {
            UniversalisStatusValue::Queued => return "Queued...".into(),
            UniversalisStatusValue::Cleanup => return "Cleaning up...".into(),
            UniversalisStatusValue::Finished => return "Finished".into(),
            UniversalisStatusValue::Processing(ids) => ids,
        };

        let proc_status = processor.status();

        let mut queued = Vec::new();
        let mut processing = 0;
        let mut done = 0;
        for id in &ids {
            match proc_status.get(id) {
                Some(AsyncProcessStatus::Active) => processing += 1,
                Some(AsyncProcessStatus::Queued(index)) => queued.push(index),
                None => done += 1,
            }
        }

        if processing > 0 {
            format!("Processing: {}, Done: {}/{}", processing, done, ids.len())
        } else if queued.is_empty() {
            "Cleaning up...".into()
        } else {
            let min_index = queued.into_iter().min().unwrap();
            format!("Queued @ {min_index}")
        }
    }
}
