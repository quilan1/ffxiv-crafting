use std::{collections::HashMap, sync::Arc};

use async_processor::{AmValue, AsyncProcessor};
use ffxiv_universalis::UniversalisHandle;

#[derive(Clone)]
pub struct MarketState {
    pub async_processor: AsyncProcessor,
    requests: AmValue<HashMap<String, UniversalisHandle>>,
}

impl MarketState {
    pub fn new() -> Arc<Self> {
        // Universalis can only take 8 connections at a time
        Arc::new(Self {
            async_processor: AsyncProcessor::new(8),
            requests: AmValue::new(HashMap::new()),
        })
    }

    pub fn async_processor(&self) -> AsyncProcessor {
        self.async_processor.clone()
    }

    pub(super) fn insert_market_request<S: AsRef<str>>(&self, uuid: S, info: UniversalisHandle) {
        let mut requests = self.requests.lock();
        requests.entry(uuid.as_ref().into()).or_insert(info);
    }

    pub(super) fn remove_market_request<S: AsRef<str>>(
        &self,
        uuid: S,
    ) -> Option<UniversalisHandle> {
        let mut requests = self.requests.lock();
        requests.remove(uuid.as_ref())
    }

    pub(super) fn with_market_request<S: AsRef<str>, F, T>(&self, uuid: S, func: F) -> T
    where
        F: Fn(Option<&mut UniversalisHandle>) -> T,
    {
        let mut requests = self.requests.lock();
        func(requests.get_mut(uuid.as_ref()))
    }
}
