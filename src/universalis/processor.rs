use super::{builder::UniversalisBuilder, json::UniversalisJson, MarketItemInfoMap};
use crate::util::{AsyncProcessor, SharedFuture};

use futures::{future::join_all, FutureExt};
use log::{error, info, warn};

#[derive(Clone)]
pub struct UniversalisRequest {
    listing: String,
    history: String,
    world: String,
}

pub struct UniversalisProcessor;

impl UniversalisProcessor {
    pub async fn process_ids<'a>(
        listing_processor: AsyncProcessor<'a>,
        builder: &UniversalisBuilder,
        ids: Vec<u32>,
    ) -> MarketItemInfoMap {
        let worlds = builder.data_centers.clone();

        let futures = Self::make_requests(listing_processor, &worlds, ids);

        let outputs = join_all(futures)
            .await
            .into_iter()
            .filter_map(|output| output)
            .collect::<Vec<_>>();

        let mut mb_info_map = MarketItemInfoMap::new();
        for output in outputs {
            info!(
                "[process_ids] world: {}, listing: {}, history: {}",
                output.world,
                output.listing.len(),
                output.history.len()
            );

            if let Err(_) =
                UniversalisJson::parse(&output.listing, &output.history, &mut mb_info_map)
            {
                error!("[process_ids] Error: Invalid json response");
            }
        }

        mb_info_map
    }

    fn make_requests<'a>(
        processor: AsyncProcessor<'a>,
        worlds: &Vec<String>,
        ids: Vec<u32>,
    ) -> Vec<SharedFuture<'a, Option<UniversalisRequest>>> {
        let mut requests = Vec::new();

        let max_chunks = ((ids.len() + 99) / 100) * worlds.len();
        let mut chunk_id = 1;
        for ids in ids.chunks(100) {
            let ids = if ids.len() != 1 {
                ids.to_vec()
            } else {
                let mut new_ids = ids.to_vec();
                new_ids.push(2);
                new_ids
            };

            let ids = ids
                .into_iter()
                .map(|id| id.to_string())
                .collect::<Vec<_>>()
                .join(",");

            for world in worlds {
                let future = UniversalisRequest::fetch(
                    processor.clone(),
                    world.clone(),
                    ids.clone(),
                    chunk_id,
                    max_chunks,
                )
                .boxed()
                .shared();
                requests.push(future);
                chunk_id += 1;
            }
        }

        requests
    }
}

impl UniversalisRequest {
    async fn fetch(
        processor: AsyncProcessor<'_>,
        world: String,
        ids: String,
        chunk_id: usize,
        max_chunks: usize,
    ) -> Option<Self> {
        async fn get(url: &str) -> Option<String> {
            Some(reqwest::get(url).await.ok()?.text().await.ok()?)
        }

        async fn fetch_listing(
            num_attempts: usize,
            world: String,
            ids: String,
            chunk_id: usize,
            max_chunks: usize,
        ) -> Option<String> {
            let listing_url = get_listing_url(world, ids);
            info!("[Fetch {chunk_id}/{max_chunks}] {listing_url}");

            for attempt in 0..num_attempts {
                let listing = get(&listing_url).await?;

                if !is_valid_json(&listing) {
                    warn!(
                        "[Fetch {chunk_id}/{max_chunks}] [{attempt}] Invalid listing json: {listing_url}"
                    );
                    continue;
                }

                info!("[Fetch {chunk_id}/{max_chunks}] Listing done");
                return Some(listing);
            }

            error!("[Fetch {chunk_id}/{max_chunks}] Failed to fetch: {listing_url}");
            return None;
        }

        async fn fetch_history(
            num_attempts: usize,
            world: String,
            ids: String,
            chunk_id: usize,
            max_chunks: usize,
        ) -> Option<String> {
            let history_url = get_history_url(world, ids);
            info!("[Fetch {chunk_id}/{max_chunks}] {history_url}");

            for attempt in 0..num_attempts {
                let history = get(&history_url).await?;

                if !is_valid_json(&history) {
                    warn!(
                        "[Fetch {chunk_id}/{max_chunks}] [{attempt}] Invalid history json: {history_url}"
                    );
                    continue;
                }

                info!("[Fetch {chunk_id}/{max_chunks}] History done");
                return Some(history);
            }

            error!("[Fetch {chunk_id}/{max_chunks}] Failed to fetch: {history_url}");
            return None;
        }

        let mut requests = Vec::new();
        requests.push(
            fetch_listing(10, world.clone(), ids.clone(), chunk_id, max_chunks)
                .boxed()
                .shared(),
        );
        requests.push(
            fetch_history(10, world.clone(), ids.clone(), chunk_id, max_chunks)
                .boxed()
                .shared(),
        );

        let mut results = processor.process(requests).await;
        let listing_result = results.remove(0);
        let history_result = results.remove(0);

        if let Some(listing) = listing_result {
            if let Some(history) = history_result {
                return Some(UniversalisRequest {
                    listing: listing,
                    history: history,
                    world,
                });
            }
        }

        None
    }
}

fn get_listing_url<S: AsRef<str>>(world: S, ids: S) -> String {
    format!(
        "https://universalis.app/api/v2/{}/{}?entries=0",
        world.as_ref(),
        ids.as_ref()
    )
}

fn get_history_url<S: AsRef<str>>(world: S, ids: S) -> String {
    format!(
        "https://universalis.app/api/v2/history/{}/{}",
        world.as_ref(),
        ids.as_ref()
    )
}

fn is_valid_json<S: AsRef<str>>(value: S) -> bool {
    let value = value.as_ref();
    value.starts_with("{") && value.ends_with("}") && value.len() > 100
}
