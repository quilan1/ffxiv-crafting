use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures::{stream::FuturesUnordered, Future, FutureExt, Stream, StreamExt};

use crate::{
    universalis::{RequestHandle, RequestResult},
    AsyncProcessorHandle, ListingsMap,
};

////////////////////////////////////////////////////////////

pub struct RequestPacket(pub RequestHandle, pub RequestHandle);

pub struct PacketRequestResult(pub RequestResult, pub RequestResult);

pub struct AsyncPacket(PacketData, PacketData);

pub struct PacketGroup(FuturesUnordered<AsyncPacket>);

/// Marketplace data for the requested items.
pub struct ListingsResults {
    /// Current marketboard listings for the associated item IDs
    pub listings: ListingsMap,
    /// Past sell data for the associated item IDs
    pub history: ListingsMap,
    /// Any ids that were unable to be fetched from Universalis
    pub failures: Vec<u32>,
}

enum PacketData {
    Handle(AsyncProcessorHandle<RequestResult>),
    Result(RequestResult),
    Done,
}

/// The state of the packet request, sent to the Universalis server.
pub enum PacketResult {
    /// The packet succeeded with first-argument current listings, second-argument history.
    Success(ListingsMap, ListingsMap),
    /// The packet failed to fetch either the current listings or the history.
    Failure(Vec<u32>),
}

////////////////////////////////////////////////////////////

impl AsyncPacket {
    pub fn new(
        listings_async: AsyncProcessorHandle<RequestResult>,
        history_async: AsyncProcessorHandle<RequestResult>,
    ) -> Self {
        Self(
            PacketData::Handle(listings_async),
            PacketData::Handle(history_async),
        )
    }
}

fn poll_handle_to_result(data: &mut PacketData, cx: &mut Context<'_>) {
    if let PacketData::Handle(ref mut handle) = data {
        if let Poll::Ready(result) = handle.poll_unpin(cx) {
            *data = PacketData::Result(result)
        }
    }
}

fn extract_result(data: &mut PacketData) -> RequestResult {
    let PacketData::Result(res) = std::mem::replace(data, PacketData::Done) else {
        unreachable!()
    };
    res
}

impl Future for AsyncPacket {
    type Output = PacketRequestResult;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        poll_handle_to_result(&mut self.0, cx);
        poll_handle_to_result(&mut self.1, cx);

        match (&self.0, &self.1) {
            (PacketData::Result(_), PacketData::Result(_)) => {}
            _ => return Poll::Pending,
        }

        // They're both results, so let's migrate them to done, and return the result
        let listings_result = extract_result(&mut self.0);
        let history_result = extract_result(&mut self.1);
        Poll::Ready(PacketRequestResult(listings_result, history_result))
    }
}

impl PacketGroup {
    pub fn new(async_packets: Vec<AsyncPacket>) -> Self {
        Self(FuturesUnordered::from_iter(async_packets))
    }

    pub async fn collect(mut self) -> ListingsResults {
        fn merge_map(dest: &mut ListingsMap, src: ListingsMap) {
            for (id, listings) in src {
                dest.entry(id).or_default().extend(listings);
            }
        }

        let mut failures = Vec::new();
        let mut listing_map = ListingsMap::new();
        let mut history_map = ListingsMap::new();
        while let Some(result) = self.next().await {
            match result {
                PacketResult::Failure(ids) => failures.extend(ids),
                PacketResult::Success(listings, history) => {
                    merge_map(&mut listing_map, listings);
                    merge_map(&mut history_map, history);
                }
            }
        }

        ListingsResults {
            listings: listing_map,
            history: history_map,
            failures,
        }
    }

    fn combine_listings(results: Vec<RequestResult>) -> PacketResult {
        let mut listing_map = ListingsMap::new();
        let mut history_map = ListingsMap::new();
        for result in results {
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
                RequestResult::Failure(ids) => return PacketResult::Failure(ids),
            }
        }
        PacketResult::Success(listing_map, history_map)
    }
}

impl Stream for PacketGroup {
    type Item = PacketResult;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.0.poll_next_unpin(cx) {
            Poll::Ready(Some(result)) => {
                Poll::Ready(Some(Self::combine_listings(vec![result.0, result.1])))
            }
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}
