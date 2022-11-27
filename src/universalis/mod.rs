use std::collections::BTreeMap;

mod builder;
mod json;
mod processor;

pub use builder::UniversalisBuilder;
pub use processor::{UniversalisAsyncProcessor, UniversalisProcessor};
use serde::Serialize;

//////////////////////////////////////////////////////

// Directly exported as json
#[derive(Debug, Default, Serialize)]
pub struct ItemListing {
    pub price: u32,
    pub count: u32,
    pub is_hq: bool,
    pub world: String,
    pub name: String,
    pub posting: u64,
}

#[derive(Debug, Default)]
pub struct MarketItemInfo {
    pub listings: Vec<ItemListing>,
    pub history: Vec<ItemListing>,
}

pub type MarketItemInfoMap = BTreeMap<u32, MarketItemInfo>;
