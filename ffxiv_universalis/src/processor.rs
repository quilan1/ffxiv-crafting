use std::time::Duration;

use crate::{
    AsyncProcessType, AsyncProcessor, FetchListingType, ItemListingMap, UniversalisStatus,
};

use futures::{future::join_all, FutureExt};
use itertools::Itertools;
use log::{error, info, warn};
use tokio::time::sleep;

const MAX_CHUNK_SIZE: usize = 100;

pub struct UniversalisProcessor {
    processor: AsyncProcessor,
    worlds: Vec<String>,
    ids: Vec<u32>,
}

////////////////////////////////////////////////////////////

impl UniversalisProcessor {
    pub fn new(processor: AsyncProcessor, worlds: Vec<String>, ids: Vec<u32>) -> Self {
        Self {
            processor,
            worlds,
            ids,
        }
    }

    pub async fn process_listings<T: FetchListingType + 'static>(
        self,
        status: UniversalisStatus,
    ) -> (ItemListingMap, Vec<u32>) {
        let results = self._process_listings::<T>(status.clone()).await;

        let mut failure_ids = Vec::new();
        let mut listing_map = ItemListingMap::new();
        for (result, ids) in results.into_iter().zip(self.id_chunks()) {
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

        status.set_finished();
        info!("[process_{}] Done!", T::fetch_type());

        let failure_ids = failure_ids.into_iter().unique().collect::<Vec<_>>();
        (listing_map, failure_ids)
    }

    async fn _process_listings<T: FetchListingType + 'static>(
        &self,
        status: UniversalisStatus,
    ) -> Vec<Option<ItemListingMap>> {
        let max_chunks = self.max_chunks();
        let id_chunks = self.id_chunks();

        let mut chunk_id = 1;
        let mut futures = Vec::new();
        for ids in &id_chunks {
            for world in &self.worlds {
                let ids_string = ids.iter().map(|id| id.to_string()).join(",");

                futures.push(
                    process_listing::<T>(
                        self.processor.clone(),
                        world.clone(),
                        ids_string,
                        status.clone(),
                        chunk_id,
                        max_chunks,
                    )
                    .boxed(),
                );
                chunk_id += 1;
            }
        }

        join_all(futures).await
    }

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

////////////////////////////////////////////////////////////

// Uses the AsyncProcessor to queue the listing & history API calls to Universalis. Once
// they return, it yields the full request back.
async fn process_listing<T: FetchListingType + 'static>(
    mut processor: AsyncProcessor,
    world: String,
    ids: String,
    status: UniversalisStatus,
    chunk_id: usize,
    max_chunks: usize,
) -> Option<ItemListingMap> {
    let future = fetch_listing::<T>(
        T::url(&world, &ids),
        format!("{}/{}", chunk_id, max_chunks),
        status.clone(),
        max_chunks,
    );

    processor
        .process_future(future.boxed(), AsyncProcessType::Limited)
        .await
}

// Grab the JSON string from a listing API from Universalis
async fn fetch_listing<T: FetchListingType + 'static>(
    url: String,
    signature: String,
    status: UniversalisStatus,
    max_chunks: usize,
) -> Option<ItemListingMap> {
    let fetch_type = T::fetch_type();
    let num_attempts = 10;
    info!("[Fetch {signature}] {url}");

    status.try_set_count(max_chunks);

    for attempt in 0..num_attempts {
        let listing = reqwest::get(&url).await.ok()?.text().await.ok()?;

        // Invalid response from the server. This typically is from load, so let's fall back a bit & retry in a second
        if !is_valid_json(&listing) {
            warn!("[Fetch {signature}] [{attempt}] Invalid {fetch_type} json: {url}");
            sleep(Duration::from_millis(500)).await;
            continue;
        }

        info!("[Fetch {signature}] {fetch_type} done");
        status.dec_count();
        return T::parse_json(listing, 7.0 * 28.0).ok();
    }

    error!("[Fetch {signature}] Failed to fetch: {url}");
    status.dec_count();
    None
}

// A dirty quick-test if something's valid json, without actually doing a full parse on it
// This is just to rule out empty json & weird server responses
fn is_valid_json<S: AsRef<str>>(value: S) -> bool {
    let value = value.as_ref();
    value.starts_with('{') && value.ends_with('}') && value.len() > 100
}

////////////////////////////////////////////////////////////
