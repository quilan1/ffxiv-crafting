use std::collections::BTreeMap;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct MultipleListingView {
    pub items: BTreeMap<String, ListingView>,
}

#[derive(Debug, Deserialize)]
pub struct MultipleHistoryView {
    pub items: BTreeMap<String, HistoryView>,
}

#[derive(Debug, Deserialize)]
pub struct ListingView {
    pub listings: Vec<ItemListingView>,
}

#[derive(Debug, Deserialize)]
pub struct HistoryView {
    pub entries: Vec<ItemListingView>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemListingView {
    pub price_per_unit: u32,
    pub hq: bool,
    pub quantity: u32,
    pub last_review_time: Option<u64>,
    pub timestamp: Option<u64>,
    pub world_name: Option<String>,
    pub retainer_name: Option<String>,
}
