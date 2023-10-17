use std::time::Duration;

use async_processor::{AmValue, AsyncProcessor};
use futures::FutureExt;
use tokio::time::sleep;

use crate::{Signal, UniversalisRequestHandle, MAX_UNIVERSALIS_CONCURRENT_FUTURES};

#[derive(Clone)]
pub struct UniversalisStatus(AmValue<UniversalisStatusData>);

pub struct UniversalisStatusData {
    async_processor: AsyncProcessor,
    state: UniversalisStatusState,
}

pub enum UniversalisStatusState {
    Queued,
    Processing(Vec<UniversalisRequestHandle>),
    Cleanup,
    Finished,
}

pub enum UniversalisStatusValues {
    Text(String),
    Processing(Vec<UniversalisProcessorState>),
}

pub enum UniversalisProcessorState {
    Active,
    Warn,
    Finished(bool),
    Queued(i32),
}

impl UniversalisStatus {
    pub fn new(async_processor: AsyncProcessor) -> Self {
        Self(AmValue::new(UniversalisStatusData {
            async_processor,
            state: UniversalisStatusState::Queued,
        }))
    }

    pub(crate) fn set_value(&self, value: UniversalisStatusState) {
        self.0.lock().state = value;
    }

    pub async fn signals(&self) -> (Vec<Signal<()>>, Vec<Signal<bool>>) {
        loop {
            {
                let state = &self.0.lock().state;
                match state {
                    UniversalisStatusState::Processing(handles) => {
                        return handles
                            .iter()
                            .map(|handle| {
                                (handle.signal_active.clone(), handle.signal_finished.clone())
                            })
                            .unzip();
                    }
                    UniversalisStatusState::Cleanup | UniversalisStatusState::Finished => {
                        return (Vec::new(), Vec::new())
                    }
                    UniversalisStatusState::Queued => {}
                }
            }

            sleep(Duration::from_millis(10)).await;
        }
    }

    pub fn values(&self) -> UniversalisStatusValues {
        use UniversalisProcessorState as P;
        use UniversalisStatusValues as V;

        let data = self.0.lock();
        let state = &data.state;
        let async_processor = &data.async_processor;

        let handles = match state {
            UniversalisStatusState::Queued => return V::Text("Queued...".into()),
            UniversalisStatusState::Cleanup => return V::Text("Cleaning up...".into()),
            UniversalisStatusState::Finished => return V::Text("Done".into()),
            UniversalisStatusState::Processing(handles) => handles,
        };

        let queue_offset = async_processor.num_finished() + MAX_UNIVERSALIS_CONCURRENT_FUTURES;

        V::Processing(
            handles
                .iter()
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
