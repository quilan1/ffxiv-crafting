use async_processor::{AmValue, AsyncProcessor};

#[derive(Clone)]
pub struct UniversalisStatus {
    data: AmValue<UniversalisStatusData>,
}

#[derive(Clone)]
pub struct UniversalisStatusData {
    #[allow(dead_code)]
    async_processor: AsyncProcessor,
    num_futures: usize,
    num_active: usize,
    num_processed: usize,
    state: UniversalisStatusState,
}

#[derive(Clone)]
pub enum UniversalisStatusState {
    Queued,
    Processing,
    Cleanup,
    Finished,
}

impl UniversalisStatus {
    pub fn new(async_processor: AsyncProcessor, num_futures: usize) -> Self {
        Self {
            data: AmValue::new(UniversalisStatusData {
                async_processor,
                num_futures,
                num_active: 0,
                num_processed: 0,
                state: UniversalisStatusState::Queued,
            }),
        }
    }

    pub fn get_num_futures(&self) -> usize {
        self.data.lock().num_futures
    }

    pub(crate) fn set_value(&self, value: UniversalisStatusState) {
        self.data.lock().state = value;
    }

    pub(crate) fn start_future(&self) {
        self.data.lock().num_active += 1;
    }

    pub(crate) fn finish_future(&self) {
        let mut data = self.data.lock();
        data.num_active -= 1;
        data.num_processed += 1;
    }

    pub fn text(&self) -> String {
        let UniversalisStatusData {
            state,
            num_futures,
            num_active,
            num_processed,
            ..
        } = self.data.lock().clone();
        match state {
            UniversalisStatusState::Queued => return "Queued...".into(),
            UniversalisStatusState::Cleanup => return "Cleaning up...".into(),
            UniversalisStatusState::Finished => return "Finished".into(),
            UniversalisStatusState::Processing => {}
        };

        if num_processed == num_futures {
            "Cleaning up...".into()
        } else if num_active > 0 || num_processed > 0 {
            format!("Processing: {num_active}, Done: {num_processed}/{num_futures}")
        } else {
            // TODO: Expose further information via queue position
            "Queued".into()
        }
    }
}
