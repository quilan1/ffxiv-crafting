use std::sync::Arc;

use parking_lot::Mutex;

use crate::{universalis::AsyncProcessor, MReceiver, RequestState};

use super::{RequestPacket, MAX_UNIVERSALIS_CONCURRENT_FUTURES};

////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct StatusController(Arc<Mutex<StatusControllerData>>);

struct StatusControllerData {
    async_processor: AsyncProcessor,
    packets: Vec<RequestPacket>,
}

pub enum Status {
    Text(String),
    Processing(Vec<FetchState>),
}

pub enum FetchState {
    Active,
    Warn,
    Finished(bool),
    Queued(i32),
}

////////////////////////////////////////////////////////////

impl StatusController {
    pub fn new(async_processor: AsyncProcessor) -> Self {
        Self(Arc::new(Mutex::new(StatusControllerData {
            async_processor,
            packets: Vec::new(),
        })))
    }

    pub(crate) fn set_packets(&self, packets: Vec<RequestPacket>) {
        self.0.lock().packets = packets;
    }

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

    pub fn values(&self) -> Status {
        use FetchState as P;
        use Status as V;

        let data = self.0.lock();
        let async_processor = &data.async_processor;
        let packets = &data.packets;
        let queue_offset = async_processor.num_finished() + MAX_UNIVERSALIS_CONCURRENT_FUTURES;

        V::Processing(
            packets
                .iter()
                .flat_map(|packet| [&packet.0, &packet.1])
                .map(|handle| match handle.state_receiver.get() {
                    RequestState::Finished(status) => P::Finished(status),
                    RequestState::Warn => P::Warn,
                    RequestState::Active => P::Active,
                    RequestState::Queued => {
                        P::Queued((handle.id as i32 - queue_offset as i32 + 1).max(0))
                    }
                })
                .collect(),
        )
    }
}
