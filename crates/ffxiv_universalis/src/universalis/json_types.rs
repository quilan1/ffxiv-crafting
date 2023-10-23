#![doc(hidden)]

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct MultipleListingView {
    pub items: BTreeMap<String, ListingView>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MultipleHistoryView {
    pub items: BTreeMap<String, HistoryView>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ListingView {
    pub listings: Vec<ItemListingView>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct HistoryView {
    pub entries: Vec<ItemListingView>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemListingView {
    pub price_per_unit: u32,
    pub hq: bool,
    pub quantity: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_review_time: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub world_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retainer_name: Option<String>,
}
