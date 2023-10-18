use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures::{channel::oneshot::Receiver, Future, FutureExt};
use tokio::task::{JoinError, JoinHandle};

use crate::universalis::ListingsMap;

use super::StatusController;

////////////////////////////////////////////////////////////

pub struct ProcessorHandleOutput {
    pub listings: ListingsMap,
    pub history: ListingsMap,
    pub failure_ids: Vec<u32>,
}

pub struct ProcessorHandle {
    uuid: String,
    join_handle: JoinHandle<ProcessorHandleOutput>,
    status: StatusController,
    ready_signal: Option<Receiver<()>>,
}

////////////////////////////////////////////////////////////

impl ProcessorHandle {
    pub(crate) fn new(
        uuid: String,
        join_handle: JoinHandle<ProcessorHandleOutput>,
        status: StatusController,
        spawn_signal: Receiver<()>,
    ) -> Self {
        Self {
            uuid,
            join_handle,
            status,
            ready_signal: Some(spawn_signal),
        }
    }

    pub fn status(&self) -> StatusController {
        self.status.clone()
    }

    pub fn uuid(&self) -> &str {
        &self.uuid
    }

    pub async fn wait_for_ready(&mut self) {
        if let Some(signal) = self.ready_signal.take() {
            signal.await.unwrap();
        }
    }
}

impl Drop for ProcessorHandle {
    fn drop(&mut self) {
        log::info!(target: "ffxiv_universalis", "{} Handle dropped", self.uuid);
        self.join_handle.abort();
    }
}

impl Future for ProcessorHandle {
    type Output = Result<ProcessorHandleOutput, JoinError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.join_handle.poll_unpin(cx)
    }
}
