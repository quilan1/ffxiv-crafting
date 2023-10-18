use async_processor::AsyncProcessor;
use futures::{
    channel::oneshot::{self, Sender},
    future::join_all,
};
use itertools::Itertools;
use mock_traits::FileDownloader;
use tokio::task::spawn_blocking;

use crate::universalis::{ListingsMap, Request, RequestResult, RequestType};

use super::{
    AsyncPacket, ProcessorData, ProcessorHandle, ProcessorHandleOutput, ProcessorInternalState,
    RequestPacket,
};

////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct Processor {
    async_processor: AsyncProcessor,
}

////////////////////////////////////////////////////////////

pub const MAX_UNIVERSALIS_CONCURRENT_FUTURES: usize = 8;

impl Processor {
    pub fn new() -> Self {
        Self {
            async_processor: AsyncProcessor::new(MAX_UNIVERSALIS_CONCURRENT_FUTURES),
        }
    }

    pub fn async_processor(&self) -> AsyncProcessor {
        self.async_processor.clone()
    }

    pub fn make_request<F: FileDownloader>(
        &self,
        worlds: &[String],
        ids: &[u32],
        retain_num_days: f32,
    ) -> ProcessorHandle {
        let data = ProcessorData::new(self.async_processor(), worlds, ids, retain_num_days);
        let status = data.status.clone();
        let uuid = data.uuid.clone();

        let (ready_signal_tx, ready_signal_rx) = oneshot::channel();
        let join_handle = tokio::spawn(async move {
            let status = data.status.clone();
            let uuid = data.uuid.clone();

            log::info!(target: "ffxiv_universalis", "{uuid} Queueing futures");
            let all_listings =
                Self::fetch_and_process_market_info::<F>(data, ready_signal_tx).await;
            status.set_value(ProcessorInternalState::Cleanup);

            let (listing_map, history_map, failure_ids) =
                spawn_blocking(move || Self::combine_returned_listings(all_listings))
                    .await
                    .unwrap();

            status.set_value(ProcessorInternalState::Finished);
            log::info!(target: "ffxiv_universalis", "{uuid} Process all done!");

            let failure_ids = failure_ids.into_iter().unique().collect::<Vec<_>>();
            ProcessorHandleOutput {
                listings: listing_map,
                history: history_map,
                failure_ids,
            }
        });

        ProcessorHandle::new(uuid, join_handle, status, ready_signal_rx)
    }

    fn combine_returned_listings(
        all_listings: Vec<RequestResult>,
    ) -> (ListingsMap, ListingsMap, Vec<u32>) {
        let mut failure_ids = Vec::new();
        let mut listing_map = ListingsMap::new();
        let mut history_map = ListingsMap::new();
        for result in all_listings {
            match result {
                RequestResult::Listing(listings) => {
                    listings.into_iter().for_each(|(key, mut listings)| {
                        listing_map.entry(key).or_default().append(&mut listings);
                    });
                }
                RequestResult::History(listings) => {
                    listings.into_iter().for_each(|(key, mut listings)| {
                        history_map.entry(key).or_default().append(&mut listings);
                    });
                }
                RequestResult::Failure(ids) => failure_ids.append(&mut ids.clone()),
            }
        }
        (listing_map, history_map, failure_ids)
    }

    async fn fetch_and_process_market_info<F: FileDownloader>(
        data: ProcessorData,
        ready_signal: Sender<()>,
    ) -> Vec<RequestResult> {
        let id_chunks = data.id_chunks();

        let mut chunk_id = 1;
        let mut handles = Vec::new();
        for ids in &id_chunks {
            for world in &data.worlds {
                let listings = Request::<F>::new(
                    data.clone(),
                    world.clone(),
                    ids.clone(),
                    RequestType::Listing,
                    chunk_id,
                )
                .process_listing();

                let history = Request::<F>::new(
                    data.clone(),
                    world.clone(),
                    ids.clone(),
                    RequestType::History,
                    chunk_id,
                )
                .process_listing();

                handles.push((listings, history));
                chunk_id += 1;
            }
        }

        let _ = ready_signal.send(());

        let mut async_handles = Vec::new();
        let mut request_handles = Vec::new();
        for ((listings_async, listings_req), (history_async, history_req)) in handles {
            async_handles.push(AsyncPacket::new(listings_async, history_async));
            request_handles.push(RequestPacket(listings_req, history_req));
        }

        data.status
            .set_value(ProcessorInternalState::Processing(request_handles));

        join_all(async_handles)
            .await
            .into_iter()
            .flat_map(|packet| [packet.0, packet.1])
            .collect_vec()
    }
}

impl Default for Processor {
    fn default() -> Self {
        Self::new()
    }
}
