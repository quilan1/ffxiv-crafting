use std::sync::Arc;

use parking_lot::Mutex;

use crate::{universalis::AsyncProcessor, MReceiver, RequestState};

use super::{RequestPacket, MAX_UNIVERSALIS_CONCURRENT_FUTURES};

////////////////////////////////////////////////////////////

/// Struct that allows access to the the current progress of a universalis processor request.
#[derive(Clone)]
pub struct Status(Arc<Mutex<StatusData>>);

struct StatusData {
    async_processor: AsyncProcessor,
    packets: Vec<RequestPacket>,
}

////////////////////////////////////////////////////////////

impl Status {
    pub(crate) fn new(async_processor: AsyncProcessor) -> Self {
        Self(Arc::new(Mutex::new(StatusData {
            async_processor,
            packets: Vec::new(),
        })))
    }

    pub(crate) fn set_packets(&self, packets: Vec<RequestPacket>) {
        self.0.lock().packets = packets;
    }

    /// Returns receivers for every fetch request sent to the async processor.
    pub async fn signals(&self) -> Vec<MReceiver<RequestState>> {
        let packets = &self.0.lock().packets;
        packets
            .iter()
            .flat_map(|packet| {
                [
                    packet.0.state_receiver.clone(),
                    packet.1.state_receiver.clone(),
                ]
            })
            .collect()
    }

    /// Returns the current state of all of the fetch requests sent to the async processor.
    pub fn values(&self) -> Vec<RequestState> {
        use RequestState as R;

        let data = self.0.lock();
        let async_processor = &data.async_processor;
        let packets = &data.packets;
        let queue_offset = async_processor.num_finished() + MAX_UNIVERSALIS_CONCURRENT_FUTURES;

        packets
            .iter()
            .flat_map(|packet| [&packet.0, &packet.1])
            .map(|handle| match handle.state_receiver.get() {
                RequestState::Queued(_) => {
                    R::Queued((handle.id as i32 - queue_offset as i32 + 1).max(0))
                }
                v => v,
            })
            .collect()
    }
}
