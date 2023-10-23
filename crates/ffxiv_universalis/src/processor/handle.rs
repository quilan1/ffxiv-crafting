use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures::{Stream, StreamExt};

use crate::ListingsResults;

use super::{
    packet::{PacketGroup, PacketResult},
    Status,
};

////////////////////////////////////////////////////////////

/// An opaque structure used to keep track of requests from Universalis.
///
/// Results may be returned either incrementally, by await individual packet
/// data (via the [Stream] trait) or by calling [collect_all()](#method.collect_all)
/// to combine all of the results together.
///
/// If this structure is dropped, before awaiting the results, all of the
/// current associated outgoing futures will be dropped as they are polled
/// internally.
pub struct ProcessorHandle {
    uuid: String,
    packet_group: PacketGroup,
    status: Status,
}

////////////////////////////////////////////////////////////

impl ProcessorHandle {
    pub(crate) fn new(uuid: String, packet_group: PacketGroup, status: Status) -> Self {
        Self {
            uuid,
            packet_group,
            status,
        }
    }

    /// The current status of the processor handle.
    pub fn status(&self) -> Status {
        self.status.clone()
    }

    /// A unique identifier for the current universalis requests. Seen in logging.
    pub fn uuid(&self) -> &str {
        &self.uuid
    }

    /// Waits for all of the results to come back from Universalis and then gathers them together
    /// into a [ListingsResults] struct.
    pub async fn collect_all(&mut self) -> ListingsResults {
        let packet_group = std::mem::replace(&mut self.packet_group, PacketGroup::new(Vec::new()));
        packet_group.collect().await
    }
}

impl Drop for ProcessorHandle {
    fn drop(&mut self) {
        log::info!(target: "ffxiv_universalis", "{} Handle dropped", self.uuid);
    }
}

impl Stream for ProcessorHandle {
    type Item = PacketResult;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.packet_group.poll_next_unpin(cx)
    }
}
