use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures::{Stream, StreamExt};

use crate::ListingsResults;

use super::{
    packet::{PacketGroup, PacketResult},
    StatusController,
};

////////////////////////////////////////////////////////////

pub struct ProcessorHandle {
    uuid: String,
    packet_group: PacketGroup,
    status: StatusController,
}

////////////////////////////////////////////////////////////

impl ProcessorHandle {
    pub(crate) fn new(uuid: String, packet_group: PacketGroup, status: StatusController) -> Self {
        Self {
            uuid,
            packet_group,
            status,
        }
    }

    pub fn status(&self) -> StatusController {
        self.status.clone()
    }

    pub fn uuid(&self) -> &str {
        &self.uuid
    }

    pub async fn collect(&mut self) -> ListingsResults {
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
