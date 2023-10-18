use std::time::Duration;

use async_processor::{AmValue, AsyncProcessor};
use futures::FutureExt;
use tokio::time::sleep;

use crate::Signal;

use super::{RequestPacket, MAX_UNIVERSALIS_CONCURRENT_FUTURES};

////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct StatusController(AmValue<StatusControllerData>);

struct StatusControllerData {
    async_processor: AsyncProcessor,
    state: ProcessorInternalState,
}

pub enum ProcessorInternalState {
    Queued,
    Processing(Vec<RequestPacket>),
    Cleanup,
    Finished,
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
        Self(AmValue::new(StatusControllerData {
            async_processor,
            state: ProcessorInternalState::Queued,
        }))
    }

    pub(crate) fn set_value(&self, value: ProcessorInternalState) {
        self.0.lock().state = value;
    }

    pub async fn signals(&self) -> (Vec<Signal<()>>, Vec<Signal<bool>>) {
        loop {
            {
                let state = &self.0.lock().state;
                match state {
                    ProcessorInternalState::Processing(packets) => {
                        return packets
                            .iter()
                            .flat_map(|packet| {
                                [
                                    (
                                        packet.0.signal_active.clone(),
                                        packet.0.signal_finished.clone(),
                                    ),
                                    (
                                        packet.1.signal_active.clone(),
                                        packet.1.signal_finished.clone(),
                                    ),
                                ]
                            })
                            .unzip();
                    }
                    ProcessorInternalState::Cleanup | ProcessorInternalState::Finished => {
                        return (Vec::new(), Vec::new())
                    }
                    ProcessorInternalState::Queued => {}
                }
            }

            sleep(Duration::from_millis(10)).await;
        }
    }

    pub fn values(&self) -> Status {
        use FetchState as P;
        use Status as V;

        let data = self.0.lock();
        let state = &data.state;
        let async_processor = &data.async_processor;

        let packets = match state {
            ProcessorInternalState::Queued => return V::Text("Queued...".into()),
            ProcessorInternalState::Cleanup => return V::Text("Cleaning up...".into()),
            ProcessorInternalState::Finished => return V::Text("Done".into()),
            ProcessorInternalState::Processing(packets) => packets,
        };

        let queue_offset = async_processor.num_finished() + MAX_UNIVERSALIS_CONCURRENT_FUTURES;

        V::Processing(
            packets
                .iter()
                .flat_map(|packet| [&packet.0, &packet.1])
                .map(|handle| {
                    if let Some(Ok(status)) = handle.signal_finished.clone().now_or_never() {
                        P::Finished(status)
                    } else if handle.signal_warn.clone().now_or_never().is_some() {
                        P::Warn
                    } else if handle.signal_active.clone().now_or_never().is_some() {
                        P::Active
                    } else {
                        P::Queued((handle.id as i32 - queue_offset as i32 + 1).max(0))
                    }
                })
                .collect(),
        )
    }
}
