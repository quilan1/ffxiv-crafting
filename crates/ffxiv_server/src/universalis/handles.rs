use std::{
    collections::BTreeMap,
    time::{Duration, Instant},
};

use anyhow::Result;
use axum::extract::ws::WebSocket;
use ffxiv_universalis::{
    MReceiver, PacketResult, Processor, ProcessorHandle, RequestBuilder, RequestState,
};
use futures::{future::BoxFuture, stream::FuturesUnordered, FutureExt, StreamExt};
use mock_traits::FileDownloader;
use tokio::{select, sync::broadcast::Receiver, time::sleep};

use super::{write_message, DetailedStatus, Input, Output};

////////////////////////////////////////////////////////////

const DUR_MIN_WAIT: Duration = Duration::from_millis(10);
const DUR_TIMEOUT: Duration = Duration::from_millis(10000);

pub async fn wait_for_universalis<F: FileDownloader>(
    socket: &mut WebSocket,
    universalis_processor: &Processor,
    payload: Input,
    all_ids: &[u32],
    server_uuid: &str,
) -> Result<()> {
    let mut request_stream =
        make_market_request_info::<F>(universalis_processor, payload, all_ids, server_uuid).await;

    // Send over initial messages
    request_stream.send_finished_packets(socket).await?;
    request_stream.send_status_update(socket).await?;
    request_stream.retain_fresh_signals();

    // Wait until we've finished all history and listings
    while !request_stream.is_done() {
        if let Some(None) = socket.recv().now_or_never() {
            log::info!(target: "ffxiv_server", "{server_uuid} WebSocket closed unexpectedly");
            break;
        }

        request_stream.wait_for_update(DUR_TIMEOUT).await;
        request_stream.send_finished_packets(socket).await?;
        request_stream.send_status_update(socket).await?;
        request_stream.retain_fresh_signals();
    }

    request_stream.send_finished_packets(socket).await?;
    log::info!(target: "ffxiv_server", "{server_uuid} WebSocket done!");

    Ok(())
}

async fn make_market_request_info<F: FileDownloader>(
    universalis_processor: &Processor,
    payload: Input,
    all_ids: &[u32],
    server_uuid: &str,
) -> RequestStream {
    let handle = RequestBuilder::new(all_ids, payload.purchase_from.clone())
        .sell_to(payload.sell_to.clone())
        .retain_num_days(payload.retain_num_days.unwrap_or(7.0))
        .execute::<F>(universalis_processor);
    log::info!(target: "ffxiv_server",
        "Server uuid {server_uuid} maps to universalis uuid {}",
        handle.uuid()
    );

    let signals = handle.status().signals().await;
    RequestStream::new(handle, signals, payload.is_compressed.unwrap_or(false))
}

////////////////////////////////////////////////////////////

struct RequestStream {
    handle: ProcessorHandle,
    values: BTreeMap<usize, MReceiver<RequestState>>,
    futures: FuturesUnordered<BoxFuture<'static, usize>>,
    last_update: Instant,
    is_compressed: bool,
}

////////////////////////////////////////////////////////////

impl RequestStream {
    fn new(
        handle: ProcessorHandle,
        signals: Vec<MReceiver<RequestState>>,
        is_compressed: bool,
    ) -> Self {
        let mut values = BTreeMap::new();
        let futures = FuturesUnordered::new();
        for (index, signal) in signals.into_iter().enumerate() {
            futures.push(Self::receiver_future(index, signal.receiver()).boxed());
            values.insert(index, signal);
        }

        Self {
            handle,
            values,
            futures,
            last_update: Instant::now(),
            is_compressed,
        }
    }

    fn is_done(&self) -> bool {
        self.values.is_empty() && self.futures.is_empty()
    }

    fn is_stale(&self, timeout: Duration) -> bool {
        self.last_update.elapsed() >= timeout
    }

    fn time_to_stale(&self, timeout: Duration) -> Duration {
        // Don't want negative values
        timeout - self.last_update.elapsed().min(timeout)
    }

    fn retain_fresh_signals(&mut self) {
        self.values
            .retain(|_, value| !matches!(value.get(), RequestState::Finished(_)));
    }

    async fn wait_for_signal(&mut self) -> bool {
        if let Some(index) = self.futures.next().await {
            if let Some(value) = self.values.get(&index) {
                return if matches!(value.get(), RequestState::Finished(_)) {
                    true
                } else {
                    self.futures
                        .push(Self::receiver_future(index, value.receiver()).boxed());
                    false
                };
            }
        }

        false
    }

    async fn receiver_future(index: usize, mut receiver: Receiver<RequestState>) -> usize {
        let _ = receiver.recv().await;
        index
    }

    async fn wait_for_update(&mut self, max_timeout: Duration) -> bool {
        let timeout = self.time_to_stale(max_timeout).max(DUR_MIN_WAIT);

        select! {
            needs_update = self.wait_for_signal() => {
                if needs_update {
                    return true;
                }
            }
            _ = sleep(timeout) => {}
        }

        self.is_stale(max_timeout)
    }

    async fn get_next_finished_packet(&mut self) -> Option<Output> {
        self.handle
            .next()
            .await
            .map(|packet_result| match packet_result {
                PacketResult::Success(listings, history) => Output::Success { listings, history },
                PacketResult::Failure(failures) => Output::Failure(failures),
            })
    }

    async fn send_finished_packets(&mut self, socket: &mut WebSocket) -> Result<()> {
        let wait = self.is_done();
        loop {
            let output = if wait {
                self.get_next_finished_packet().await
            } else {
                match self.get_next_finished_packet().now_or_never() {
                    Some(output) => output,
                    None => break,
                }
            };

            let output = output.unwrap_or(Output::Done {});
            let message_text = serde_json::to_string(&output)?;
            write_message(socket, message_text, self.is_compressed).await?;

            if matches!(output, Output::Done {}) {
                break;
            }
        }
        Ok(())
    }

    async fn send_status_update(&mut self, socket: &mut WebSocket) -> Result<()> {
        self.last_update = Instant::now();
        let values = self.handle.status().values();
        let output = Output::Status(values.into_iter().map(DetailedStatus::from).collect());

        let message_text = serde_json::to_string(&output)?;
        write_message(socket, message_text, self.is_compressed).await?;
        Ok(())
    }
}
