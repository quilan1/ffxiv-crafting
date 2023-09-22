use std::collections::BTreeMap;

use crate::{
    ItemListing, ItemMarketInfoMap, UniversalisHandle, UniversalisProcessorData,
    UniversalisRequest, UniversalisRequestType, UniversalisStatusState,
};

use async_processor::AsyncProcessor;
use futures::{
    channel::oneshot::{self, Sender},
    future::join_all,
};
use itertools::Itertools;
use tokio::task::spawn_blocking;

#[derive(Clone)]
pub struct UniversalisProcessor {
    async_processor: AsyncProcessor,
}

////////////////////////////////////////////////////////////

pub const MAX_UNIVERSALIS_CONCURRENT_FUTURES: usize = 8;

impl UniversalisProcessor {
    pub fn new() -> Self {
        Self {
            async_processor: AsyncProcessor::new(MAX_UNIVERSALIS_CONCURRENT_FUTURES),
        }
    }

    pub fn async_processor(&self) -> AsyncProcessor {
        self.async_processor.clone()
    }

    pub fn make_request<T: UniversalisRequestType>(
        &self,
        worlds: Vec<String>,
        ids: Vec<u32>,
        retain_num_days: f32,
    ) -> UniversalisHandle {
        let data =
            UniversalisProcessorData::new(self.async_processor(), worlds, ids, retain_num_days);
        let status = data.status.clone();
        let uuid = data.uuid.clone();

        let (ready_signal_tx, ready_signal_rx) = oneshot::channel();
        let join_handle = tokio::spawn(async move {
            let status = data.status.clone();
            let uuid = data.uuid.clone();
            let chunks = data.id_chunks();

            log::info!(target: "ffxiv_universalis", "{uuid} Queueing {} futures", T::fetch_type());
            let all_listings =
                Self::fetch_and_process_market_info::<T>(data, ready_signal_tx).await;
            status.set_value(UniversalisStatusState::Cleanup);

            let (listing_map, failure_ids) =
                spawn_blocking(move || Self::combine_returned_listings(chunks, all_listings))
                    .await
                    .unwrap();

            status.set_value(UniversalisStatusState::Finished);
            log::info!(target: "ffxiv_universalis", "{uuid} Process all {} done!", T::fetch_type());

            let failure_ids = failure_ids.into_iter().unique().collect::<Vec<_>>();
            (listing_map, failure_ids)
        });

        UniversalisHandle::new(uuid, join_handle, status, ready_signal_rx)
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

    async fn fetch_and_process_market_info<T: UniversalisRequestType>(
        data: UniversalisProcessorData,
        ready_signal: Sender<()>,
    ) -> Vec<Option<ItemMarketInfoMap>> {
        let id_chunks = data.id_chunks();

        let mut chunk_id = 1;
        let mut handles = Vec::new();
        for ids in &id_chunks {
            for world in &data.worlds {
                let ids_string = ids.iter().map(|id| id.to_string()).join(",");
                let request =
                    UniversalisRequest::<T>::new(data.clone(), world.clone(), ids_string, chunk_id);
                handles.push(request.process_listing());
                chunk_id += 1;
            }
        }

        let _ = ready_signal.send(());

        let (async_handles, request_handles): (Vec<_>, Vec<_>);
        (async_handles, request_handles) = handles.into_iter().unzip();

        data.status
            .set_value(UniversalisStatusState::Processing(request_handles));

        join_all(async_handles).await
    }
}

impl Default for UniversalisProcessor {
    fn default() -> Self {
        Self::new()
    }
}

////////////////////////////////////////////////////////////
