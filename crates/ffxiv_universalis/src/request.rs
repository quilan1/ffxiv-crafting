use std::{marker::PhantomData, time::Duration};

use async_processor::AsyncProcessorHandle;
use futures::{
    channel::oneshot::{self, Receiver},
    future::{BoxFuture, Shared},
    FutureExt,
};
use tokio::{task::spawn_blocking, time::sleep};

use crate::{ItemMarketInfoMap, UniversalisProcessorData, UniversalisRequestType};

////////////////////////////////////////////////////////////

pub type Signal<T> = Shared<Receiver<T>>;

pub struct UniversalisRequest<T: UniversalisRequestType, D: FileDownloader> {
    data: UniversalisProcessorData,
    url: String,
    signature: String,
    _marker: PhantomData<fn() -> T>, // Allowing T to be Send & Sync
    _marker_d: PhantomData<fn() -> D>,
}

pub struct UniversalisRequestHandle {
    pub signal_active: Signal<()>,
    pub signal_finished: Signal<bool>,
    pub id: usize,
}

////////////////////////////////////////////////////////////

impl<T: UniversalisRequestType, D: FileDownloader> UniversalisRequest<T, D> {
    pub fn new(
        data: UniversalisProcessorData,
        world: String,
        ids: String,
        request_id: usize,
    ) -> Self {
        Self {
            url: T::url(&world, &ids),
            signature: format!("{}/{}", request_id, data.num_requests),
            data,
            _marker: PhantomData,
            _marker_d: PhantomData,
        }
    }

    // Uses the AsyncProcessor to queue the listing & history API calls to Universalis. Once
    // they return, it yields the full request back.
    pub fn process_listing(
        self,
    ) -> (
        AsyncProcessorHandle<Option<ItemMarketInfoMap>>,
        UniversalisRequestHandle,
    ) {
        let async_processor = self.data.async_processor.clone();
        let (signal_active_tx, signal_active_rx) = oneshot::channel();
        let (signal_finished_tx, signal_finished_rx) = oneshot::channel::<bool>();

        let future = async move {
            let _ = signal_active_tx.send(());
            let results = Self::fetch_listing_url(
                self.data.uuid,
                self.url,
                self.signature,
                self.data.retain_num_days,
            )
            .await;
            let _ = signal_finished_tx.send(results.is_some());
            results
        };

        let async_processor_handle = async_processor.process_future(future);
        let request_handle = UniversalisRequestHandle {
            signal_active: signal_active_rx.shared(),
            signal_finished: signal_finished_rx.shared(),
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
    ) -> Option<ItemMarketInfoMap> {
        let fetch_type = T::fetch_type();
        let num_attempts = 10;
        log::info!(target: "ffxiv_universalis", "{uuid} Fetch {signature} {url}");

        for attempt in 0..num_attempts {
            let listing = D::download_file(&url).await?;

            // Invalid response from the server. This typically is from load, so let's fall back a bit & retry in a second
            if !is_valid_json(&listing) {
                log::warn!(target: "ffxiv_universalis", "{uuid} Fetch {signature} [{attempt}] Invalid {fetch_type} json: {url}");
                sleep(Duration::from_millis(1000)).await;
                continue;
            }

            log::info!(target: "ffxiv_universalis", "{uuid} Fetch {signature} {fetch_type} done");
            return spawn_blocking(move || T::parse_json(listing, retain_num_days).ok())
                .await
                .unwrap();
        }

        log::error!(target: "ffxiv_universalis", "{uuid} Fetch {signature} failed: {url}");
        None
    }
}

// A dirty quick-test if something's valid json, without actually doing a full parse on it
// This is just to rule out empty json & weird server responses
fn is_valid_json<S: AsRef<str>>(value: S) -> bool {
    let value = value.as_ref();
    value.starts_with('{') && value.ends_with('}') && value.len() > 100
}

pub trait FileDownloader {
    fn download_file(url: &str) -> BoxFuture<'_, Option<String>>;
}

pub struct ReqwestDownloader;

impl FileDownloader for ReqwestDownloader {
    fn download_file(url: &str) -> BoxFuture<'_, Option<String>> {
        async fn inner(url: &str) -> Option<String> {
            reqwest::get(url).await.ok()?.text().await.ok()
        }
        inner(url).boxed()
    }
}
