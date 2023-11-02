use mock_traits::FileDownloader;

use crate::{Processor, ProcessorHandle};

/// Struct that allows one to build a request to be fetched from Universalis.
pub struct RequestBuilder {
    ids: Vec<u32>,
    purchase_from: String,
    sell_to: Option<String>,
    retain_num_days: Option<f32>,
}

impl RequestBuilder {
    /// Creates a new request for item ids on a world/datacenter/region.
    pub fn new<S: Into<String>>(ids: &[u32], purchase_from: S) -> Self {
        Self {
            ids: ids.to_vec(),
            purchase_from: purchase_from.into(),
            sell_to: None,
            retain_num_days: None,
        }
    }

    /// Adds a location from which items are sold. This should be one's homeworld.
    pub fn sell_to<S: Into<String>>(mut self, sell_to: S) -> Self {
        self.sell_to = Some(sell_to.into());
        self
    }

    /// Adds a limit for number of date of results returned; no older sales will be returned in the request.
    pub fn retain_num_days(mut self, retain_num_days: f32) -> Self {
        self.retain_num_days = Some(retain_num_days);
        self
    }

    /// Queues a request to Universalis and returns the handle.
    pub fn execute<F: FileDownloader>(self, processor: &Processor) -> ProcessorHandle {
        processor.make_request::<F>(
            &self.ids,
            self.purchase_from.clone(),
            self.sell_to.unwrap_or(self.purchase_from.clone()),
            self.retain_num_days.unwrap_or(7.0),
        )
    }
}
