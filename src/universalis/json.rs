use anyhow::Result;
use serde::Deserialize;
use std::{
    collections::BTreeMap,
    time::{Duration, SystemTime},
};

use crate::Settings;

use super::{ItemListing, MarketBoardInfo, UniversalisRequest};

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

pub fn process_json(
    request: &UniversalisRequest,
    response: &str,
    all_mb_info: &mut BTreeMap<String, MarketBoardInfo>,
) -> Result<()> {
    let mb_entry = all_mb_info.entry(request.world.clone()).or_default();
    let listing = serde_json::from_str::<MultipleListingView>(&response)?;
    for (id, info) in &listing.items {
        let id = id.parse::<u32>()?;
        let entry = mb_entry.entry(id).or_default();
        entry.price_avg = info.averagePrice;
        entry.price_nq = info.averagePriceNQ;
        entry.price_hq = info.averagePriceHQ;
        entry.min_price_hq = info.minPriceHQ;
        (entry.velocity_nq, entry.velocity_hq) = calculate_velocity(&info.recentHistory);
        entry.listings = info
            .listings
            .iter()
            .map(|listing| ItemListing {
                price: listing.pricePerUnit,
                count: listing.quantity,
                is_hq: listing.hq,
                world: listing.worldName.clone().unwrap_or_default(),
                name: listing.retainerName.clone().unwrap_or_default(),
                posting: listing.lastReviewTime.unwrap(),
            })
            .collect::<Vec<_>>();
    }

    Ok(())
}

fn calculate_velocity(history: &Vec<ItemListingView>) -> (f32, f32) {
    if history.len() == 0 {
        return (0.0, 0.0);
    }

    let mut sold_nq = 0;
    let mut sold_hq = 0;

    let mut max_days = 0.0;
    for item in history {
        let days = SystemTime::UNIX_EPOCH + Duration::from_secs(item.timestamp.unwrap());
        let days = days.elapsed().unwrap().as_secs_f32() / (3600.0 * 24.0);
        max_days = days;
        if days > 7.0 {
            break;
        }

        match item.hq {
            false => sold_nq += item.quantity,
            true => sold_hq += item.quantity,
        }
    }

    max_days = max_days.min(7.0);
    (sold_nq as f32 / max_days, sold_hq as f32 / max_days)
}
