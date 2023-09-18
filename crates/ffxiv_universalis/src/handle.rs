use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures::{channel::oneshot::Receiver, Future, FutureExt};
use tokio::task::{JoinError, JoinHandle};

use crate::{ItemMarketInfoMap, UniversalisStatus};

pub type UniversalisHandleOutput = (ItemMarketInfoMap, Vec<u32>);

pub struct UniversalisHandle {
    uuid: String,
    join_handle: JoinHandle<UniversalisHandleOutput>,
    status: UniversalisStatus,
    ready_signal: Option<Receiver<()>>,
}

impl UniversalisHandle {
    pub(crate) fn new(
        uuid: String,
        join_handle: JoinHandle<UniversalisHandleOutput>,
        status: UniversalisStatus,
        spawn_signal: Receiver<()>,
    ) -> Self {
        Self {
            uuid,
            join_handle,
            status,
            ready_signal: Some(spawn_signal),
        }
    }

    pub fn status(&self) -> UniversalisStatus {
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

impl Drop for UniversalisHandle {
    fn drop(&mut self) {
        log::info!("[Universalis] {} Handle dropped", self.uuid);
        self.join_handle.abort();
    }
}

impl Future for UniversalisHandle {
    type Output = Result<UniversalisHandleOutput, JoinError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.join_handle.poll_unpin(cx)
    }
}
