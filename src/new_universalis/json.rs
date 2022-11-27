use anyhow::Result;
use serde::Deserialize;
use std::{
    collections::BTreeMap,
    time::{Duration, SystemTime},
};

use crate::cli::settings;

use super::{ItemListing, MarketItemInfoMap};

#[derive(Debug, Clone, Deserialize)]
struct MultipleListingView {
    items: BTreeMap<String, ListingView>,
}

#[derive(Debug, Clone, Deserialize)]
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

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code, non_snake_case)]
struct ItemListingView {
    pricePerUnit: u32,
    hq: bool,
    quantity: u32,
    lastReviewTime: Option<u64>,
    timestamp: Option<u64>,
    worldName: Option<String>,
    retainerName: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct MultipleHistoryView {
    items: BTreeMap<String, HistoryView>,
}

#[derive(Debug, Clone, Deserialize)]
struct HistoryView {
    entries: Vec<ItemListingView>,
}

pub struct UniversalisJson;

impl UniversalisJson {
    pub fn parse(
        listings_json: &str,
        history_json: &str,
        mb_info_map: &mut MarketItemInfoMap,
    ) -> Result<()> {
        let MultipleListingView { items } =
            serde_json::from_str::<MultipleListingView>(&listings_json)?;
        for (id, info) in items.into_iter() {
            let mut listings = parse_recent_listings(info.listings);

            let id = id.parse::<u32>()?;
            let entry = mb_info_map.entry(id).or_default();
            entry.listings.append(&mut listings);
            entry.listings.sort_by(|a, b| a.price.cmp(&b.price));
        }

        let MultipleHistoryView { items } =
            serde_json::from_str::<MultipleHistoryView>(&history_json)?;
        for (id, info) in items.into_iter() {
            let mut listings = parse_recent_listings(info.entries);

            let id = id.parse::<u32>()?;
            let entry = mb_info_map.entry(id).or_default();
            entry.history.append(&mut listings);
            entry.history.sort_by(|a, b| a.price.cmp(&b.price));
        }

        Ok(())
    }
}

fn parse_recent_listings(item_listing_view: Vec<ItemListingView>) -> Vec<ItemListing> {
    item_listing_view
        .into_iter()
        .filter(|listing| {
            // Only history listings have a timestamp, so limit those to the last week
            if let Some(timestamp) = listing.timestamp {
                let days = SystemTime::UNIX_EPOCH + Duration::from_secs(timestamp);
                let days = days.elapsed().unwrap().as_secs_f32() / (3600.0 * 24.0);
                days <= 7.0
            } else {
                true
            }
        })
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
        .collect::<Vec<_>>()
}
