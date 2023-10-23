use std::{marker::PhantomData, time::Duration};

use itertools::Itertools;
use mock_traits::FileDownloader;
use tokio::{task::spawn_blocking, time::sleep};

use crate::{multi_signal, processor::ProcessorData, MReceiver, MSender};

use super::{AsyncProcessorHandle, ListingsMap, RequestType};

////////////////////////////////////////////////////////////

pub struct Request<F: FileDownloader> {
    data: ProcessorData,
    url: String,
    signature: String,
    ids: Vec<u32>,
    request_type: RequestType,
    _marker_f: PhantomData<fn() -> F>, // Allows F to be Send & Sync
}

/// The current state of a request made to the universalis server.
#[derive(Clone, Debug)]
pub enum RequestState {
    /// The request is queued for processing and will begin as soon as there is capacity for it.
    Queued,
    /// The request is currently fetching data from the universalis server.
    Active,
    /// The request has failed at least once, to fetch results from universalis.
    Warn,
    /// The request has finished either successfully (true) or unsuccessfully (false).
    Finished(bool),
}

pub struct RequestHandle {
    pub state_receiver: MReceiver<RequestState>,
    pub id: usize,
}

pub enum RequestResult {
    Listing(ListingsMap),
    History(ListingsMap),
    Failure(Vec<u32>),
}

////////////////////////////////////////////////////////////

impl<F: FileDownloader> Request<F> {
    pub fn new(
        data: ProcessorData,
        world: String,
        ids: Vec<u32>,
        request_type: RequestType,
        request_id: usize,
    ) -> Self {
        let ids_string = ids.iter().map(|id| id.to_string()).join(",");
        Self {
            url: request_type.url(&world, &ids_string),
            signature: format!("{}/{}", request_id, data.num_requests),
            data,
            ids,
            request_type,
            _marker_f: PhantomData,
        }
    }

    // Uses the AsyncProcessor to queue the listing & history API calls to Universalis. Once
    // they return, it yields the full request back.
    pub fn process_listing(self) -> (AsyncProcessorHandle<RequestResult>, RequestHandle) {
        let async_processor = self.data.async_processor.clone();
        let (state_sender, state_receiver) = multi_signal(RequestState::Queued, 4);

        let future = async move {
            let results = Self::fetch_listing_url(
                self.data.uuid,
                self.url,
                self.signature,
                self.data.retain_num_days,
                self.request_type.clone(),
                state_sender,
            )
            .await;

            match results {
                Some(listings) => self.request_type.result_listings(listings),
                None => RequestResult::Failure(self.ids),
            }
        };

        let async_processor_handle = async_processor.process_future(future);
        let request_handle = RequestHandle {
            state_receiver,
            id: async_processor_handle.id(),
        };

        (async_processor_handle, request_handle)
    }

    // Grab the JSON string from a listing API from Universalis
    async fn fetch_listing_url(
        uuid: String,
        url: String,
        signature: String,
        retain_num_days: f32,
        request_type: RequestType,
        mut state_sender: MSender<RequestState>,
    ) -> Option<ListingsMap> {
        let fetch_type = request_type.fetch_type();
        let num_attempts = 10;
        log::info!(target: "ffxiv_universalis", "{uuid} Fetch {signature} {url}");
        let _ = state_sender.set(RequestState::Active);

        for attempt in 0..num_attempts {
            let listing = F::download(&url).await.ok()?;

            // Invalid response from the server. This typically is from load, so let's fall back a bit & retry in a second
            if !is_valid_json(&listing) {
                log::warn!(target: "ffxiv_universalis", "{uuid} Fetch {signature} [{attempt}] Invalid {fetch_type} json: {url}");
                if attempt == 0 {
                    let _ = state_sender.set(RequestState::Warn);
                }
                sleep(Duration::from_millis(1000)).await;
                continue;
            }

            log::info!(target: "ffxiv_universalis", "{uuid} Fetch {signature} {fetch_type} done");
            let _ = state_sender.set(RequestState::Finished(true));
            return spawn_blocking(move || request_type.parse_json(listing, retain_num_days).ok())
                .await
                .unwrap();
        }

        log::error!(target: "ffxiv_universalis", "{uuid} Fetch {signature} failed: {url}");
        let _ = state_sender.set(RequestState::Finished(false));
        None
    }
}

// A dirty quick-test if something's valid json, without actually doing a full parse on it
// This is just to rule out empty json & weird server responses
fn is_valid_json<S: AsRef<str>>(value: S) -> bool {
    let value = value.as_ref();
    value.starts_with('{') && value.ends_with('}') && value.len() > 100
}
