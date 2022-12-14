use std::{fmt::Display, time::Duration};

use super::{json::UniversalisJson, MarketItemInfoMap};
use crate::util::{AmValue, AsyncProcessor, ProcessType};

use futures::{future::join_all, join, Future, FutureExt};
use log::{error, info, warn};
use tokio::time::sleep;

const MAX_CHUNK_SIZE: usize = 100;

struct UniversalisRequest {
    listing: String,
    history: String,
}

#[derive(Clone)]
pub struct UniversalisStatus {
    data: AmValue<UniversalisStatusValue>,
}

#[derive(Clone)]
enum UniversalisStatusValue {
    Queued,
    Remaining(usize),
    Processing,
    Finished,
}

pub struct UniversalisProcessor;

impl UniversalisProcessor {
    // Takes all of the IDs given, sends them out to Universalis and collates the results
    pub async fn process_ids(
        processor: AsyncProcessor,
        worlds: Vec<String>,
        ids: Vec<u32>,
        retain_num_days: f32,
        status: UniversalisStatus,
    ) -> MarketItemInfoMap {
        let requests = Self::make_requests(processor, worlds, ids, status.clone()).await;
        status.set_processing();

        let mut mb_info_map = MarketItemInfoMap::new();
        for request in requests {
            if UniversalisJson::parse(
                &request.listing,
                &request.history,
                &mut mb_info_map,
                retain_num_days,
            )
            .is_err()
            {
                error!("[process_ids] Error: Invalid json response");
            }
        }

        status.set_finished();
        info!("[process_ids] Done!");
        mb_info_map
    }

    // Processes the ids by creating futures of the fetch requests in a big pool, and awaiting them all
    // Universalis accepts up to 100 IDs in a single request, so we chunk them up as such, avoiding
    // the case where there is one single ID, by adding on a harmless one at the end.
    async fn make_requests(
        processor: AsyncProcessor,
        worlds: Vec<String>,
        ids: Vec<u32>,
        status: UniversalisStatus,
    ) -> Vec<UniversalisRequest> {
        let mut requests = Vec::new();

        let max_chunks = ((ids.len() + MAX_CHUNK_SIZE - 1) / MAX_CHUNK_SIZE) * worlds.len();

        let mut chunk_id = 1;
        for ids in Self::chunk_ids(&ids) {
            for world in &worlds {
                requests.push(
                    UniversalisRequest::fetch(
                        processor.clone(),
                        world.clone(),
                        ids.clone(),
                        status.clone(),
                        chunk_id,
                        max_chunks,
                    )
                    .boxed(),
                );
                chunk_id += 1;
            }
        }

        join_all(requests).await.into_iter().flatten().collect()
    }

    // Return the chunks as a comma-delimited string of 100 ids (or whatever remains)
    fn chunk_ids(ids: &[u32]) -> Vec<String> {
        let mut id_chunks = Vec::new();
        for ids in ids.chunks(MAX_CHUNK_SIZE) {
            let ids = if ids.len() == 1 {
                // If there's only one ID in the group, the json will be different, so to make it a
                // multiple-id request, we just tack on the id #2, 'Fire Shard'
                let mut new_ids = ids.to_vec();
                new_ids.push(2);
                new_ids
            } else {
                ids.to_vec()
            };

            id_chunks.push(
                ids.into_iter()
                    .map(|id| id.to_string())
                    .collect::<Vec<_>>()
                    .join(","),
            );
        }

        id_chunks
    }
}

impl UniversalisRequest {
    // Uses the AsyncProcessor to queue the listing & history API calls to Universalis. Once
    // they return, it yields the full request back.
    async fn fetch(
        mut processor: AsyncProcessor,
        world: String,
        ids: String,
        status: UniversalisStatus,
        chunk_id: usize,
        max_chunks: usize,
    ) -> Option<Self> {
        #[allow(clippy::unused_async)]
        async fn try_set_status(status: UniversalisStatus, max_chunks: usize) {
            status.try_set_count(max_chunks);
        }

        let init_status = try_set_status(status.clone(), max_chunks).boxed().shared();

        let signature_listing = format!("{}/{}", 2 * chunk_id - 1, 2 * max_chunks);
        let signature_history = format!("{}/{}", 2 * chunk_id, 2 * max_chunks);
        let listing_url = get_listing_url(&world, &ids);
        let history_url = get_history_url(&world, &ids);

        let listing = fetch_listing(
            "listing".into(),
            listing_url,
            signature_listing,
            init_status.clone(),
        )
        .boxed();
        let history = fetch_listing(
            "history".into(),
            history_url,
            signature_history,
            init_status,
        )
        .boxed();
        let listing = processor.process_future(listing, ProcessType::Limited);
        let history = processor.process_future(history, ProcessType::Limited);
        let (listing_result, history_result) = join!(listing, history);

        status.dec_count();

        if let Some(listing) = listing_result {
            if let Some(history) = history_result {
                return Some(UniversalisRequest { listing, history });
            }
        }

        None
    }
}

// Some utility functions for the status, to prevent accidentally deadlocking via the mutex (oops)
impl UniversalisStatus {
    pub fn new() -> Self {
        Self {
            data: AmValue::with_value(UniversalisStatusValue::Queued),
        }
    }

    #[allow(dead_code)]
    pub fn is_finished(&self) -> bool {
        let data = self.data.lock();
        data.is_finished()
    }

    fn try_set_count(&self, count: usize) {
        let mut data = self.data.lock();
        if let UniversalisStatusValue::Queued = *data {
            *data = UniversalisStatusValue::Remaining(count);
        }
    }

    fn dec_count(&self) {
        let mut data = self.data.lock();
        if let UniversalisStatusValue::Remaining(count) = *data {
            *data = UniversalisStatusValue::Remaining(count - 1);
        }
    }

    fn set_processing(&self) {
        let mut data = self.data.lock();
        *data = UniversalisStatusValue::Processing;
    }

    fn set_finished(&self) {
        let mut data = self.data.lock();
        *data = UniversalisStatusValue::Finished;
    }
}

impl Display for UniversalisStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let data = self.data.lock();
        write!(f, "{data}")
    }
}

#[allow(dead_code)]
impl UniversalisStatusValue {
    fn is_finished(&self) -> bool {
        matches!(*self, UniversalisStatusValue::Finished)
    }
}

// String value for the status
impl Display for UniversalisStatusValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            UniversalisStatusValue::Queued => write!(f, "Queued..."),
            UniversalisStatusValue::Remaining(count) => write!(f, "Remaining: {count}"),
            UniversalisStatusValue::Processing => write!(f, "Processing..."),
            UniversalisStatusValue::Finished => write!(f, "Finished"),
        }
    }
}

// Grab the JSON string from a listing API from Universalis
async fn fetch_listing<Fut>(
    fetch_type: String,
    url: String,
    signature: String,
    init_status: Fut,
) -> Option<String>
where
    Fut: Future,
{
    let num_attempts = 10;
    info!("[Fetch {signature}] {url}");

    init_status.await;

    for attempt in 0..num_attempts {
        let listing = reqwest::get(&url).await.ok()?.text().await.ok()?;

        // Invalid response from the server. This typically is from load, so let's fall back a bit & retry in a second
        if !is_valid_json(&listing) {
            warn!("[Fetch {signature}] [{attempt}] Invalid {fetch_type} json: {url}");
            sleep(Duration::from_millis(500)).await;
            continue;
        }

        info!("[Fetch {signature}] {fetch_type} done");
        return Some(listing);
    }

    error!("[Fetch {signature}] Failed to fetch: {url}");
    None
}

// Universalis API for buy listings
fn get_listing_url<S: AsRef<str>>(world: S, ids: S) -> String {
    format!(
        "https://universalis.app/api/v2/{}/{}?entries=0",
        world.as_ref(),
        ids.as_ref()
    )
}

// Universalis API for sell history listings
fn get_history_url<S: AsRef<str>>(world: S, ids: S) -> String {
    format!(
        "https://universalis.app/api/v2/history/{}/{}",
        world.as_ref(),
        ids.as_ref()
    )
}

// A dirty quick-test if something's valid json, without actually doing a full parse on it
// This is just to rule out empty json & weird server responses
fn is_valid_json<S: AsRef<str>>(value: S) -> bool {
    let value = value.as_ref();
    value.starts_with('{') && value.ends_with('}') && value.len() > 100
}
