use std::{collections::HashMap, sync::Arc};

use async_processor::{AmValue, AsyncProcessor};
use ffxiv_universalis::{UniversalisHandle, UniversalisProcessor};

#[derive(Clone)]
pub struct MarketState {
    pub processor: UniversalisProcessor,
    handles: AmValue<HashMap<String, UniversalisHandle>>,
}

impl MarketState {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            processor: UniversalisProcessor::new(),
            handles: AmValue::new(HashMap::new()),
        })
    }

    pub fn async_processor(&self) -> AsyncProcessor {
        self.processor.async_processor()
    }

    pub fn processor(&self) -> UniversalisProcessor {
        self.processor.clone()
    }

    pub(super) fn insert_handle<S: AsRef<str>>(&self, uuid: S, info: UniversalisHandle) {
        let mut requests = self.handles.lock();
        requests.entry(uuid.as_ref().into()).or_insert(info);
    }

    pub(super) fn remove_handle<S: AsRef<str>>(&self, uuid: S) -> Option<UniversalisHandle> {
        let mut requests = self.handles.lock();
        requests.remove(uuid.as_ref())
    }

    pub(super) fn with_handle<S: AsRef<str>, F, T>(&self, uuid: S, func: F) -> T
    where
        F: Fn(Option<&mut UniversalisHandle>) -> T,
    {
        let mut requests = self.handles.lock();
        func(requests.get_mut(uuid.as_ref()))
    }
}
