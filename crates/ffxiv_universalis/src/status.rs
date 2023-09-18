use async_processor::{AmValue, AsyncProcessor, AsyncProcessorStatus};

use crate::MAX_UNIVERSALIS_CONCURRENT_FUTURES;

#[derive(Clone)]
pub struct UniversalisStatus(AmValue<UniversalisStatusData>);

pub struct UniversalisStatusData {
    async_processor: AsyncProcessor,
    state: UniversalisStatusState,
}

pub enum UniversalisStatusState {
    Queued,
    Processing(Vec<usize>),
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

    pub fn text(&self) -> String {
        let (proc_num_finished, ids) = {
            let data = self.0.lock();
            let state = &data.state;
            let async_processor = &data.async_processor;

            let ids = match state {
                UniversalisStatusState::Queued => return "Queued...".into(),
                UniversalisStatusState::Cleanup => return "Cleaning up...".into(),
                UniversalisStatusState::Finished => return "Done".into(),
                UniversalisStatusState::Processing(ids) => ids,
            };

            (async_processor.num_finished(), ids.clone())
        };

        let statuses = ids
            .iter()
            .map(|&id| {
                let offset = id as i32 - proc_num_finished as i32;
                if offset < 0 {
                    AsyncProcessorStatus::Done
                } else if offset < MAX_UNIVERSALIS_CONCURRENT_FUTURES as i32 {
                    AsyncProcessorStatus::Active
                } else {
                    AsyncProcessorStatus::Queued(
                        offset as usize - MAX_UNIVERSALIS_CONCURRENT_FUTURES,
                    )
                }
            })
            .collect::<Vec<_>>();

        let num_futures = statuses.len();

        let mut num_done = 0;
        let mut num_active = 0;
        let mut min_queue = None;

        for status in statuses {
            match status {
                AsyncProcessorStatus::Done => num_done += 1,
                AsyncProcessorStatus::Active => num_active += 1,
                AsyncProcessorStatus::Queued(position) => {
                    min_queue = Some(min_queue.map_or(position, |prev: usize| prev.min(position)))
                }
            }
        }

        if num_done == num_futures {
            "Cleaning up...".into()
        } else {
            let active_queued = if num_active > 0 {
                format!("Active: {num_active}")
            } else {
                min_queue.map_or(String::from("Queued??"), |position| {
                    format!("Queued #{}", position + 1)
                })
            };
            format!("Done: {num_done}/{num_futures}, {active_queued}")
        }
    }
}
