use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    time::{Duration, SystemTime},
};

use super::json_types::{
    HistoryView, ItemListingView, ListingView, MultipleHistoryView, MultipleListingView,
};

////////////////////////////////////////////////////////////

/// The listing information returned from the server. Each listing represents either someone
/// selling an item, or previously having bought an item.
#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemListing {
    /// The gil price of the item listed or sold. This includes the tax!
    pub price: u32,
    /// The number of items listed or sold.
    pub count: u32,
    /// If the item in the listing is high quality.
    pub is_hq: bool,
    /// The world this is sold from. If the world given to the request is not a data center,
    /// this value will not be serialized.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub world: String,
    /// The name of the retainer selling this (listing) or character buying this (history).
    #[serde(skip_serializing_if = "String::is_empty")]
    pub name: String,
    /// Time in days since this was either updated (listing) or purchased (history).
    pub days_since: f32,
}

/// Associative map from item_ids to listings (either buying or selling) from universalis.
pub type ListingsMap = BTreeMap<u32, Vec<ItemListing>>;

////////////////////////////////////////////////////////////

pub struct UniversalisJson;

impl UniversalisJson {
    pub fn parse_listing(json: String, retain_num_days: f32) -> Result<ListingsMap> {
        Self::parse_general_listing::<MultipleListingView, ListingView>(&json, retain_num_days)
    }

    pub fn parse_history(json: String, retain_num_days: f32) -> Result<ListingsMap> {
        Self::parse_general_listing::<MultipleHistoryView, HistoryView>(&json, retain_num_days)
    }

    fn parse_general_listing<
        'a,
        MultipleView: ItemsMapTrait<String, View> + Deserialize<'a>,
        View: GeneralListingsTrait,
    >(
        json: &'a str,
        retain_num_days: f32,
    ) -> Result<ListingsMap> {
        let json_map = serde_json::from_str::<MultipleView>(json)?.items();

        let mut map = ListingsMap::new();
        for (id, mut info) in json_map {
            info.retain_recent_listings(retain_num_days);
            let mut listings = info.into_item_listings();

            let id = id.parse::<u32>()?;
            let entry = map.entry(id).or_default();
            entry.append(&mut listings);
            entry.sort_by(|a, b| a.price.cmp(&b.price));
        }

        Ok(map)
    }
}

////////////////////////////////////////////////////////////

trait ItemsMapTrait<K, V> {
    fn items(self) -> BTreeMap<K, V>;
}

trait GeneralListingsTrait
where
    Self: Sized,
{
    fn items(self) -> Vec<ItemListingView>;
    fn retain_recent_listings(&mut self, _retain_num_days: f32) {}
    fn into_item_listings(self) -> Vec<ItemListing> {
        self.items()
            .into_iter()
            .map(|listing| ItemListing {
                price: listing.price_per_unit,
                count: listing.quantity,
                is_hq: listing.hq,
                world: listing.world_name.unwrap_or_default(),
                name: listing.retainer_name.unwrap_or_default(),
                days_since: posting_days(
                    listing
                        .last_review_time
                        .unwrap_or(listing.timestamp.unwrap_or_default()),
                ),
            })
            .collect()
    }
}

////////////////////////////////////////////////////////////

impl ItemsMapTrait<String, HistoryView> for MultipleHistoryView {
    fn items(self) -> BTreeMap<String, HistoryView> {
        self.items.into_iter().collect()
    }
}

impl GeneralListingsTrait for HistoryView {
    fn items(self) -> Vec<ItemListingView> {
        self.entries
    }

    fn retain_recent_listings(&mut self, retain_num_days: f32) {
        self.entries
            .retain(|listing| posting_days(listing.timestamp.unwrap()) <= retain_num_days);
    }
}

fn posting_days(timestamp: u64) -> f32 {
    let epoch = SystemTime::UNIX_EPOCH.elapsed().unwrap().as_secs_f32();
    let stamp = Duration::from_secs(timestamp).as_secs_f32();
    (epoch - stamp) / (3600.0 * 24.0)
}

////////////////////////////////////////////////////////////

impl ItemsMapTrait<String, ListingView> for MultipleListingView {
    fn items(self) -> BTreeMap<String, ListingView> {
        self.items.into_iter().collect()
    }
}

impl GeneralListingsTrait for ListingView {
    fn items(self) -> Vec<ItemListingView> {
        self.listings
    }
}
