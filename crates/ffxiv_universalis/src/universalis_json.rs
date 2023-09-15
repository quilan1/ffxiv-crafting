use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    time::{Duration, SystemTime},
};

#[derive(Debug, Default, Serialize)]
pub struct ItemListing {
    pub price: u32,
    pub count: u32,
    pub is_hq: bool,
    pub world: String,
    pub name: String,
    pub posting: u64,
}

#[derive(Debug, Deserialize)]
struct MultipleListingView {
    items: BTreeMap<String, ListingView>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code, non_snake_case)]
struct ListingView {
    itemID: u32,
    averagePrice: f32,
    averagePriceNQ: f32,
    averagePriceHQ: f32,
    minPriceHQ: u32,
    listings: Vec<ItemListingView>,
    recentHistory: Vec<ItemListingView>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct ItemListingView {
    pricePerUnit: u32,
    hq: bool,
    quantity: u32,
    lastReviewTime: Option<u64>,
    timestamp: Option<u64>,
    worldName: Option<String>,
    retainerName: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MultipleHistoryView {
    items: BTreeMap<String, HistoryView>,
}

#[derive(Debug, Deserialize)]
struct HistoryView {
    entries: Vec<ItemListingView>,
}

pub type ItemListingMap = BTreeMap<u32, Vec<ItemListing>>;

////////////////////////////////////////////////////////////

pub struct UniversalisJson;

impl UniversalisJson {
    pub fn parse_listing(json: String, retain_num_days: f32) -> Result<ItemListingMap> {
        Self::parse_general_listing::<MultipleListingView, ListingView>(&json, retain_num_days)
    }

    pub fn parse_history(json: String, retain_num_days: f32) -> Result<ItemListingMap> {
        Self::parse_general_listing::<MultipleHistoryView, HistoryView>(&json, retain_num_days)
    }

    fn parse_general_listing<
        'a,
        MultipleView: ItemsMapTrait<String, View> + Deserialize<'a>,
        View: GeneralListingsTrait,
    >(
        json: &'a str,
        retain_num_days: f32,
    ) -> Result<ItemListingMap> {
        let json_map = serde_json::from_str::<MultipleView>(json)?.items();

        let mut map = ItemListingMap::new();
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
                price: listing.pricePerUnit,
                count: listing.quantity,
                is_hq: listing.hq,
                world: listing.worldName.unwrap_or_default(),
                name: listing.retainerName.unwrap_or_default(),
                posting: listing
                    .lastReviewTime
                    .unwrap_or(listing.timestamp.unwrap_or_default()),
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
        self.entries.retain(|listing| {
            let timestamp = listing.timestamp.unwrap();
            let days = SystemTime::UNIX_EPOCH + Duration::from_secs(timestamp);
            let days = days.elapsed().unwrap().as_secs_f32() / (3600.0 * 24.0);
            days <= retain_num_days
        });
    }
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
