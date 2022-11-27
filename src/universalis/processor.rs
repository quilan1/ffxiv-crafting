use super::{builder::UniversalisBuilder, json::UniversalisJson, MarketItemInfoMap};
use crate::util::{AsyncProcessor, SharedFuture};

use futures::FutureExt;
use log::{error, info, warn};

type UniversalisFutureOutput = Option<UniversalisRequest>;
type UniversalisRequestFuture<'a> = SharedFuture<'a, UniversalisFutureOutput>;
pub type UniversalisAsyncProcessor<'a> = AsyncProcessor<'a, UniversalisFutureOutput>;

#[derive(Clone)]
pub struct UniversalisRequest {
    listing: String,
    listing_url: String,
    history: String,
    history_url: String,
    world: String,
}

pub struct UniversalisProcessor;

impl UniversalisProcessor {
    pub async fn process_ids(
        processor: UniversalisAsyncProcessor<'_>,
        builder: &UniversalisBuilder,
        ids: Vec<u32>,
    ) -> MarketItemInfoMap {
        let worlds = builder.data_centers.clone();

        let futures = Self::make_requests(&worlds, ids);
        let outputs = processor.process(futures).await;

        let outputs = outputs
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
                error!(
                    "Error: Invalid json response for {} or {}",
                    output.listing_url, output.history_url
                );
            }
        }

        mb_info_map
    }

    fn make_requests<'a>(worlds: &Vec<String>, ids: Vec<u32>) -> Vec<UniversalisRequestFuture<'a>> {
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
                let future =
                    UniversalisRequest::fetch(world.clone(), ids.clone(), chunk_id, max_chunks)
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
    async fn fetch(world: String, ids: String, chunk_id: usize, max_chunks: usize) -> Option<Self> {
        async fn get(url: &str) -> Option<String> {
            Some(reqwest::get(url).await.ok()?.text().await.ok()?)
        }

        async fn fetch_listing(
            num_attempts: usize,
            world: &str,
            ids: &str,
            chunk_id: usize,
            max_chunks: usize,
        ) -> Option<(String, String)> {
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

                return Some((listing_url, listing));
            }

            error!("[Fetch {chunk_id}/{max_chunks}] Failed to fetch: {listing_url}");
            return None;
        }

        async fn fetch_history(
            num_attempts: usize,
            world: &str,
            ids: &str,
            chunk_id: usize,
            max_chunks: usize,
        ) -> Option<(String, String)> {
            let history_url = get_history_url(world, ids);
            info!("== [Fetch {chunk_id}/{max_chunks}] {history_url} ==");

            for attempt in 0..num_attempts {
                let history = get(&history_url).await?;

                if !is_valid_json(&history) {
                    warn!(
                        "== [Fetch {chunk_id}/{max_chunks}] [{attempt}] Invalid history json: {history_url} =="
                    );
                    continue;
                }

                return Some((history_url, history));
            }

            error!("== [Fetch {chunk_id}/{max_chunks}] Failed to fetch: {history_url} ==");
            return None;
        }

        if let Some((listing_url, listing)) =
            fetch_listing(10, &world, &ids, chunk_id, max_chunks).await
        {
            if let Some((history_url, history)) =
                fetch_history(10, &world, &ids, chunk_id, max_chunks).await
            {
                return Some(UniversalisRequest {
                    listing,
                    listing_url,
                    history,
                    history_url,
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
    value.starts_with("{") && value.ends_with("}") && value.len() > 50
}
