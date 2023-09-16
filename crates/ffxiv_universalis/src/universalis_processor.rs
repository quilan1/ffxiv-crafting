use std::{collections::BTreeMap, time::Duration};

use crate::{
    ItemListing, ItemMarketInfoMap, MarketRequestType, UniversalisStatus, UniversalisStatusValue,
};

use async_processor::{AsyncProcessor, IdFuture, SyncBoxFuture};
use futures::future::join_all;
use itertools::Itertools;
use log::{error, info, warn};
use tokio::{task::spawn, time::sleep};

const MAX_CHUNK_SIZE: usize = 100;

pub struct UniversalisProcessor;

pub struct UniversalisProcessorData {
    processor: AsyncProcessor,
    worlds: Vec<String>,
    ids: Vec<u32>,
    retain_num_days: f32,
    status: UniversalisStatus,
}

////////////////////////////////////////////////////////////

impl UniversalisProcessorData {
    fn max_chunks(&self) -> usize {
        ((self.ids.len() + MAX_CHUNK_SIZE - 1) / MAX_CHUNK_SIZE) * self.worlds.len()
    }

    // Return the chunks of 100 ids (or whatever remains)
    fn id_chunks(&self) -> Vec<Vec<u32>> {
        let mut id_chunks = Vec::new();
        for ids in self.ids.chunks(MAX_CHUNK_SIZE) {
            let ids = if ids.len() == 1 {
                // If there's only one ID in the group, the json will be different, so to make it a
                // multiple-id request, we just tack on the id #2, 'Fire Shard'
                let mut new_ids = ids.to_vec();
                new_ids.push(2);
                new_ids
            } else {
                ids.to_vec()
            };

            id_chunks.push(ids);
        }

        id_chunks
    }
}

impl UniversalisProcessor {
    pub fn market_info<T: MarketRequestType + 'static>(
        processor: AsyncProcessor,
        worlds: Vec<String>,
        ids: Vec<u32>,
        retain_num_days: f32,
    ) -> (
        SyncBoxFuture<(ItemMarketInfoMap, Vec<u32>)>,
        UniversalisStatus,
    ) {
        let status = UniversalisStatus::new();
        let data = UniversalisProcessorData {
            processor,
            worlds,
            ids,
            retain_num_days,
            status: status.clone(),
        };

        let future = async move {
            let all_listings = Self::fetch_and_process_market_info::<T>(&data).await;
            data.status.set_value(UniversalisStatusValue::Cleanup);

            let chunks = data.id_chunks();
            let (listing_map, failure_ids) =
                spawn(async move { Self::combine_returned_listings(chunks, all_listings) })
                    .await
                    .unwrap();

            data.status.set_value(UniversalisStatusValue::Finished);
            info!("[process_{}] Done!", T::fetch_type());

            let failure_ids = failure_ids.into_iter().unique().collect::<Vec<_>>();
            (listing_map, failure_ids)
        };

        (Box::pin(future), status)
    }

    fn combine_returned_listings(
        chunks: Vec<Vec<u32>>,
        all_listings: Vec<Option<BTreeMap<u32, Vec<ItemListing>>>>,
    ) -> (ItemMarketInfoMap, Vec<u32>) {
        let mut failure_ids = Vec::new();
        let mut listing_map = ItemMarketInfoMap::new();
        for (result, ids) in all_listings.into_iter().zip(chunks) {
            match result {
                Some(value) => {
                    value.into_iter().for_each(|(key, mut listings)| {
                        let entry = listing_map.entry(key).or_default();
                        entry.append(&mut listings);
                    });
                }
                None => failure_ids.append(&mut ids.clone()),
            }
        }
        (listing_map, failure_ids)
    }

    async fn fetch_and_process_market_info<T: MarketRequestType + 'static>(
        data: &UniversalisProcessorData,
    ) -> Vec<Option<ItemMarketInfoMap>> {
        let max_chunks = data.max_chunks();
        let id_chunks = data.id_chunks();

        let mut chunk_id = 1;
        let mut id_futures = Vec::new();
        for ids in &id_chunks {
            for world in &data.worlds {
                let ids_string = ids.iter().map(|id| id.to_string()).join(",");

                id_futures.push(process_listing::<T>(
                    data.processor.clone(),
                    world.clone(),
                    ids_string,
                    chunk_id,
                    max_chunks,
                    data.retain_num_days,
                ));
                chunk_id += 1;
            }
        }

        let (ids, futures): (Vec<_>, Vec<_>) = id_futures
            .into_iter()
            .map(|id_future| (id_future.id, id_future.future))
            .unzip();

        data.status
            .set_value(UniversalisStatusValue::Processing(ids));
        join_all(futures).await
    }
}

////////////////////////////////////////////////////////////

// Uses the AsyncProcessor to queue the listing & history API calls to Universalis. Once
// they return, it yields the full request back.
fn process_listing<T: MarketRequestType + 'static>(
    mut processor: AsyncProcessor,
    world: String,
    ids: String,
    chunk_id: usize,
    max_chunks: usize,
    retain_num_days: f32,
) -> IdFuture<Option<ItemMarketInfoMap>> {
    let url = T::url(&world, &ids);
    let signature = format!("{}/{}", chunk_id, max_chunks);
    processor.process_future(fetch_listing_url::<T>(url, signature, retain_num_days))
}

// Grab the JSON string from a listing API from Universalis
async fn fetch_listing_url<T: MarketRequestType + 'static>(
    url: String,
    signature: String,
    retain_num_days: f32,
) -> Option<ItemMarketInfoMap> {
    let fetch_type = T::fetch_type();
    let num_attempts = 10;
    info!("[Fetch {signature}] {url}");

    for attempt in 0..num_attempts {
        let listing = reqwest::get(&url).await.ok()?.text().await.ok()?;

        // Invalid response from the server. This typically is from load, so let's fall back a bit & retry in a second
        if !is_valid_json(&listing) {
            warn!("[Fetch {signature}] [{attempt}] Invalid {fetch_type} json: {url}");
            sleep(Duration::from_millis(500)).await;
            continue;
        }

        info!("[Fetch {signature}] {fetch_type} done");
        return spawn(async move { T::parse_json(listing, retain_num_days).ok() })
            .await
            .unwrap();
    }

    error!("[Fetch {signature}] Failed to fetch: {url}");
    None
}

// A dirty quick-test if something's valid json, without actually doing a full parse on it
// This is just to rule out empty json & weird server responses
fn is_valid_json<S: AsRef<str>>(value: S) -> bool {
    let value = value.as_ref();
    value.starts_with('{') && value.ends_with('}') && value.len() > 100
}

////////////////////////////////////////////////////////////
