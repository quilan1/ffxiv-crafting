use mock_traits::FileDownloader;

use crate::universalis::{AsyncProcessor, Request, RequestType};

use super::{packet::PacketGroup, AsyncPacket, ProcessorData, ProcessorHandle, RequestPacket};

////////////////////////////////////////////////////////////

/// Allows for communicating with the Universalis server in a manner that does not overload
/// their IP connection limiting.
#[derive(Clone)]
pub struct Processor {
    async_processor: AsyncProcessor,
}

////////////////////////////////////////////////////////////

pub const MAX_UNIVERSALIS_CONCURRENT_FUTURES: usize = 8;

impl Processor {
    /// Creates a new [Processor] such that at most 8 maximum concurrent requests will be
    /// sent to Universalis at the same time. Call [makeRequest](#method.makeRequest) to fetch the market information.
    pub fn new() -> Self {
        Self {
            async_processor: AsyncProcessor::new(MAX_UNIVERSALIS_CONCURRENT_FUTURES),
        }
    }

    /// Returns the [AsyncProcessor] that will be running the requests for this [Processor].
    pub fn async_processor(&self) -> AsyncProcessor {
        self.async_processor.clone()
    }

    /// Makes (potentially many) requests to the Universalis server to begin fetching data for all of the
    /// item ids passed in, according the options presented.
    ///
    /// # Note
    ///
    /// If the returned [ProcessorHandle] is dropped, all of the current associated requests will terminate.
    ///
    /// # Important
    ///
    /// For these requests to be fulfilled, the internal [AsyncProcessor] **must** be awaited. This may be done as
    /// follows:
    ///
    /// ```rust,no_run
    /// use mock_traits::ReqwestDownloader;
    /// use ffxiv_universalis::{Processor, RequestBuilder};
    ///
    /// let processor = Processor::new();
    /// tokio::spawn(processor.async_processor());
    ///
    /// // Grab all of the Dynamis data center market board data
    /// // for the item 'Water Shard' from within the past 7 days.
    /// let world = String::from("Dynamis");
    /// let ids = [7];  // ID=7 is "Water Shard"
    /// let handle = RequestBuilder::new(&ids, world)
    ///     .execute::<ReqwestDownloader>(&processor);
    /// ```
    ///
    /// When finished with the processor, one may disconnect it. This will allow it to finish its current
    /// requests, then terminate gracefully.
    /// ```rust,no_run
    /// # use ffxiv_universalis::Processor;
    /// # #[tokio::main]
    /// # async fn main() {
    /// # let processor = Processor::new();
    /// processor.async_processor().disconnect();
    ///
    /// // Will yield when everything is terminated
    /// processor.async_processor().await;
    /// # }
    /// ```
    pub fn make_request<F: FileDownloader>(
        &self,
        ids: &[u32],
        purchase_from: String,
        sell_to: String,
        retain_num_days: f32,
    ) -> ProcessorHandle {
        let data = ProcessorData::new(
            self.async_processor(),
            ids,
            purchase_from,
            sell_to,
            retain_num_days,
        );
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
            let listings = Request::<F>::new(
                data.clone(),
                data.purchase_from.clone(),
                ids.clone(),
                RequestType::Listing,
                chunk_id,
            )
            .process_listing();

            let history = Request::<F>::new(
                data.clone(),
                data.sell_to.clone(),
                ids.clone(),
                RequestType::History,
                chunk_id,
            )
            .process_listing();

            handles.push((listings, history));
            chunk_id += 1;
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
