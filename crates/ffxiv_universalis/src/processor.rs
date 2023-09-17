use std::collections::BTreeMap;

use crate::{
    ItemListing, ItemMarketInfoMap, MarketRequestType, UniversalisHandle, UniversalisProcessorData,
    UniversalisRequest, UniversalisStatusState,
};

use async_processor::AsyncProcessor;
use futures::future::join_all;
use itertools::Itertools;
use log::info;
use tokio::task::spawn_blocking;

pub struct UniversalisProcessor;

////////////////////////////////////////////////////////////

impl UniversalisProcessor {
    pub fn market_info<T: MarketRequestType + 'static>(
        async_processor: AsyncProcessor,
        worlds: Vec<String>,
        ids: Vec<u32>,
        retain_num_days: f32,
    ) -> UniversalisHandle {
        let data =
            UniversalisProcessorData::new(async_processor.clone(), worlds, ids, retain_num_days);
        let status = data.status.clone();
        let uuid = data.uuid.clone();

        let join_handle = tokio::spawn(async move {
            let status = data.status.clone();
            let uuid = data.uuid.clone();
            let chunks = data.id_chunks();

            info!("[Universalis] {uuid} Queueing {} futures", T::fetch_type());
            let all_listings = Self::fetch_and_process_market_info::<T>(data).await;
            status.set_value(UniversalisStatusState::Cleanup);

            let (listing_map, failure_ids) =
                spawn_blocking(move || Self::combine_returned_listings(chunks, all_listings))
                    .await
                    .unwrap();

            status.set_value(UniversalisStatusState::Finished);
            info!("[Universalis] {uuid} Process all {} done!", T::fetch_type());

            let failure_ids = failure_ids.into_iter().unique().collect::<Vec<_>>();
            (listing_map, failure_ids)
        });

        UniversalisHandle::new(uuid, join_handle, status)
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
        data: UniversalisProcessorData,
    ) -> Vec<Option<ItemMarketInfoMap>> {
        let id_chunks = data.id_chunks();

        let mut chunk_id = 1;
        let mut remote_futures = Vec::new();
        for ids in &id_chunks {
            for world in &data.worlds {
                let ids_string = ids.iter().map(|id| id.to_string()).join(",");
                let request =
                    UniversalisRequest::<T>::new(data.clone(), world.clone(), ids_string, chunk_id);
                remote_futures.push(request.process_listing());
                chunk_id += 1;
            }
        }

        data.status.set_value(UniversalisStatusState::Processing);
        join_all(remote_futures).await
    }
}

////////////////////////////////////////////////////////////
