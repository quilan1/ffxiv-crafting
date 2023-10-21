use mock_traits::FileDownloader;

use crate::universalis::{AsyncProcessor, Request, RequestType};

use super::{packet::PacketGroup, AsyncPacket, ProcessorData, ProcessorHandle, RequestPacket};

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

        log::info!(target: "ffxiv_universalis", "{uuid} Queueing futures");
        let packet_group = Self::fetch_and_process_market_info::<F>(data);
        ProcessorHandle::new(uuid, packet_group, status)
    }

    fn fetch_and_process_market_info<F: FileDownloader>(data: ProcessorData) -> PacketGroup {
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

        let mut async_packets = Vec::new();
        let mut request_packets = Vec::new();
        for ((listings_async, listings_req), (history_async, history_req)) in handles {
            async_packets.push(AsyncPacket::new(listings_async, history_async));
            request_packets.push(RequestPacket(listings_req, history_req));
        }

        data.status.set_packets(request_packets);
        PacketGroup::new(async_packets)
    }
}

impl Default for Processor {
    fn default() -> Self {
        Self::new()
    }
}
