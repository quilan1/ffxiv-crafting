use std::time::Duration;

use async_processor::{AmValue, AsyncProcessor};
use futures::{channel::oneshot::Receiver, future::Shared, FutureExt};
use tokio::time::sleep;

use crate::{UniversalisRequestHandle, MAX_UNIVERSALIS_CONCURRENT_FUTURES};

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

    pub async fn signals(&self) -> Vec<Shared<Receiver<()>>> {
        loop {
            {
                let state = &self.0.lock().state;
                match state {
                    UniversalisStatusState::Processing(handles) => {
                        return handles
                            .iter()
                            .flat_map(|handle| {
                                vec![handle.signal_active.clone(), handle.signal_finished.clone()]
                            })
                            .collect();
                    }
                    UniversalisStatusState::Cleanup | UniversalisStatusState::Finished => {
                        return Vec::new()
                    }
                    UniversalisStatusState::Queued => {}
                }
            }

            sleep(Duration::from_millis(10)).await;
        }
    }

    pub fn text(&self) -> String {
        let (num_futures, num_active, num_finished, min_queue_position) = {
            let data = self.0.lock();
            let state = &data.state;
            let async_processor = &data.async_processor;

            let handles = match state {
                UniversalisStatusState::Queued => return "Queued...".into(),
                UniversalisStatusState::Cleanup => return "Cleaning up...".into(),
                UniversalisStatusState::Finished => return "Done".into(),
                UniversalisStatusState::Processing(handles) => handles,
            };

            let num_futures = handles.len();
            let (num_active, num_finished) = Self::active_finished_counts(handles);
            let min_queue_position =
                Self::min_queue_position(handles, async_processor.num_finished());
            (num_futures, num_active, num_finished, min_queue_position)
        };

        if num_finished == num_futures {
            "Cleaning up...".into()
        } else {
            let active_queued = if num_active > 0 {
                format!("Active: {num_active}")
            } else {
                min_queue_position.map_or(String::from("Queued??"), |position| {
                    format!("Queued #{}", position + 1)
                })
            };
            format!("Done: {num_finished}/{num_futures}, {active_queued}")
        }
    }

    fn active_finished_counts(handles: &[UniversalisRequestHandle]) -> (usize, usize) {
        let mut active = 0;
        let mut finished = 0;
        for handle in handles {
            let signal_active = handle.signal_active.clone().now_or_never();
            let signal_finished = handle.signal_finished.clone().now_or_never();
            active += signal_active.is_some() as usize;
            finished += signal_finished.is_some() as usize;
        }

        (active - finished, finished)
    }

    fn min_queue_position(
        handles: &[UniversalisRequestHandle],
        proc_num_finished: usize,
    ) -> Option<usize> {
        handles
            .iter()
            .map(|handle| handle.id)
            .filter(|&id| id >= proc_num_finished + MAX_UNIVERSALIS_CONCURRENT_FUTURES)
            .map(|id| id - proc_num_finished - MAX_UNIVERSALIS_CONCURRENT_FUTURES)
            .min()
    }
}
