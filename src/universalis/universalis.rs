use anyhow::Result;
use std::collections::BTreeMap;

use crate::cli::settings;
use crate::universalis::ProcessorStream;
use crate::util::library;

#[derive(Debug, Default)]
pub struct MarketBoardItemInfo {
    pub price_nq: f32,
    pub price_hq: f32,
    pub price_avg: f32,
    pub min_price_hq: u32,
    pub velocity_nq: f32,
    pub velocity_hq: f32,
    pub listings: Vec<ItemListing>,
}

#[derive(Debug, Default)]
pub struct ItemListing {
    pub price: u32,
    pub is_hq: bool,
    pub count: u32,
    pub posting: u64,
    pub world: String,
    pub name: String,
}

pub type MarketBoardInfo = BTreeMap<u32, MarketBoardItemInfo>;

pub struct Universalis {
    pub homeworld: MarketBoardInfo,
    pub data_centers: Vec<MarketBoardInfo>,
}

#[derive(Clone)]
pub struct UniversalisRequest {
    pub world: String,
    pub url: String,
    pub chunk: usize,
    pub kind: UniversalisRequestType,
}

#[derive(Clone)]
pub enum UniversalisRequestType {
    Listings,
    History,
}

impl Universalis {
    pub async fn get_mb_info() -> Result<Self> {
        Ok(Self::get_mb_info_ids(library().all_market_board_ids()).await?)
    }

    pub async fn get_mb_info_ids(ids: Vec<u32>) -> Result<Self> {
        let homeworld = settings().homeworld.as_str();
        let data_centers = settings().data_centers.iter().map(|v| v.as_str()).collect();
        let requests = Self::create_mb_requests(&ids, homeworld, &data_centers);
        println!(
            "Creating {} requests for {} items",
            requests.len(),
            ids.len()
        );

        let processor = ProcessorStream::new(requests)?;
        let mut mb_info = processor.process(homeworld, &data_centers).await;

        let homeworld = mb_info.remove(homeworld).unwrap();
        let data_center_info = data_centers
            .iter()
            .map(|&data_center| mb_info.remove(data_center).unwrap())
            .collect::<Vec<_>>();
        Ok(Self {
            homeworld: homeworld,
            data_centers: data_center_info,
        })
    }

    fn create_mb_requests(
        ids: &Vec<u32>,
        homeworld: &str,
        data_centers: &Vec<&str>,
    ) -> Vec<UniversalisRequest> {
        let mut requests = Vec::new();
        for (chunk, ids) in ids.chunks(100).enumerate() {
            let ids = ids
                .into_iter()
                .map(|id| id.to_string())
                .collect::<Vec<_>>()
                .join(",");

            let mut worlds = data_centers.clone();
            worlds.push(homeworld);

            for world in worlds {
                requests.push(UniversalisRequest {
                    world: world.into(),
                    url: get_listing_url(world, &ids),
                    chunk,
                    kind: UniversalisRequestType::Listings,
                });

                if world == homeworld {
                    requests.push(UniversalisRequest {
                        world: world.into(),
                        url: get_history_url(world, &ids),
                        chunk,
                        kind: UniversalisRequestType::History,
                    });
                }
            }
        }
        requests
    }
}

fn get_listing_url(world: &str, ids: &str) -> String {
    format!("https://universalis.app/api/v2/{world}/{ids}?entries=0")
}

fn get_history_url(world: &str, ids: &str) -> String {
    format!("https://universalis.app/api/v2/history/{world}/{ids}")
}
