use anyhow::Result;
use std::collections::BTreeMap;
use std::time::Instant;

use crate::universalis::Processor;
use crate::{library::Library, Settings};

#[derive(Debug, Clone, Default)]
pub struct MarketBoardItemInfo {
    pub price: f32,
    pub price_hq: f32,
    pub min_price_hq: u32,
    pub velocity: f32,
    pub velocity_hq: f32,
    pub listings: Vec<ItemListing>,
}

#[derive(Debug, Clone, Default)]
pub struct ItemListing {
    pub price: u32,
    pub is_hq: bool,
    pub count: u32,
    pub world: String,
}

pub type MarketBoardInfo = BTreeMap<u32, MarketBoardItemInfo>;

pub struct Universalis {
    pub homeworld: MarketBoardInfo,
    pub data_center: MarketBoardInfo,
}

#[derive(Clone)]
pub struct UniversalisRequest {
    pub world: String,
    pub url: String,
    pub chunk: usize,
}

impl Universalis {
    pub async fn get_mb_info(
        library: &Library,
        settings: &Settings,
        homeworld: &str,
        data_center: &str,
    ) -> Result<Self> {
        let ids = library.all_market_board_ids(settings);
        let requests = Self::create_mb_requests(&ids, homeworld, data_center);
        println!(
            "Creating {} requests for {} items",
            requests.len(),
            ids.len()
        );

        let start = Instant::now();
        let processor = Processor::new(requests, homeworld, data_center)?;
        let mb_info = processor.process().await?;

        println!(
            "Total time taken: {}s",
            start.elapsed().as_secs()
        );

        Ok(Self {
            homeworld: mb_info[homeworld].clone(),
            data_center: mb_info[data_center].clone(),
        })
    }

    fn create_mb_requests(
        ids: &Vec<u32>,
        homeworld: &str,
        data_center: &str,
    ) -> Vec<UniversalisRequest> {
        let mut requests = Vec::new();
        for (chunk, ids) in ids.chunks(100).enumerate() {
            let ids = ids
                .into_iter()
                .map(|id| id.to_string())
                .collect::<Vec<_>>()
                .join(",");

            for world in [homeworld, data_center] {
                requests.push(UniversalisRequest {
                    world: world.into(),
                    url: get_listing_url(world, &ids),
                    chunk,
                });
            }
        }
        requests
    }
}

fn get_listing_url(world: &str, ids: &str) -> String {
    format!("https://universalis.app/api/v2/{world}/{ids}?entries=1000")
}
