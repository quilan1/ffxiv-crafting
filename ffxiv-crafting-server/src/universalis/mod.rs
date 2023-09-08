use std::collections::BTreeMap;

mod builder;
mod gen_listing;
mod json;
mod processor;
mod status;

pub use builder::UniversalisBuilder;
pub use gen_listing::{GenListing, History, Listing};
pub use json::ItemListingMap;
pub use processor::UniversalisProcessor;
pub use status::UniversalisStatus;

////////////////////////////////////////////////////////////

use serde::Serialize;

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
