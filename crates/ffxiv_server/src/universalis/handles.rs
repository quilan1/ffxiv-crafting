use std::time::{Duration, Instant};

use anyhow::Result;
use axum::extract::ws::{Message, WebSocket};
use ffxiv_universalis::{Processor, ProcessorHandle, ProcessorHandleOutput, Signal, Status};
use futures::{Future, FutureExt};
use mock_traits::FileDownloader;
use tokio::time::sleep;

use super::{DetailedStatus, Input, Output};

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
    let mut market_request_info =
        make_market_request_info::<F>(universalis_processor, payload, all_ids, server_uuid).await;

    // Send over initial messages
    market_request_info.process_handle(socket).await?;
    market_request_info.retain_fresh_signals();

    // Wait until we've finished all history and listings
    let timeouts = [DUR_TIMEOUT, Duration::from_millis(50)];
    while !market_request_info.is_done() {
        if let Some(None) = socket.recv().now_or_never() {
            log::info!(target: "ffxiv_server", "{server_uuid} WebSocket closed unexpectedly");
            break;
        }

        let timeout = timeouts[usize::from(market_request_info.is_waiting_for_cleanup())];
        if market_request_info.wait_for_update(timeout).await {
            market_request_info.process_handle(socket).await?;
        }
        market_request_info.retain_fresh_signals();
    }
    Ok(())
}

async fn make_market_request_info<F: FileDownloader>(
    universalis_processor: &Processor,
    payload: Input,
    all_ids: &[u32],
    server_uuid: &str,
) -> MarketRequestState {
    let worlds: Vec<_> = payload
        .data_center
        .or(std::env::var("FFXIV_DATA_CENTERS").ok())
        .unwrap()
        .split(',')
        .map(str::trim)
        .map(ToString::to_string)
        .collect();

    let retain_num_days = payload.retain_num_days.unwrap_or(7.0);

    let handle = universalis_processor.make_request::<F>(&worlds, all_ids, retain_num_days);

    log::info!(target: "ffxiv_server",
        "Server uuid {server_uuid} maps to universalis uuid {}",
        handle.uuid()
    );

    let signals = handle.status().signals().await;
    MarketRequestState::new(handle, signals)
}

////////////////////////////////////////////////////////////

enum MarketRequestState {
    Processing {
        handle: ProcessorHandle,
        signals_active: Vec<Signal<()>>,
        signals_finished: Vec<Signal<bool>>,
        last_update: Instant,
    },
    Done,
}

////////////////////////////////////////////////////////////

fn is_signal_done<T>(signal: &Signal<T>) -> bool
where
    Signal<T>: Future,
{
    signal.clone().now_or_never().is_some()
}

impl MarketRequestState {
    fn new(
        handle: ProcessorHandle,
        (signals_active, signals_finished): (Vec<Signal<()>>, Vec<Signal<bool>>),
    ) -> Self {
        Self::Processing {
            handle,
            signals_active,
            signals_finished,
            last_update: Instant::now(),
        }
    }

    fn are_any_signals_done(&self) -> bool {
        match self {
            MarketRequestState::Done => false,
            MarketRequestState::Processing {
                signals_active,
                signals_finished,
                ..
            } => {
                signals_active.iter().any(is_signal_done)
                    || signals_finished.iter().any(is_signal_done)
            }
        }
    }

    fn is_done(&self) -> bool {
        matches!(self, MarketRequestState::Done)
    }

    fn is_waiting_for_cleanup(&self) -> bool {
        match self {
            MarketRequestState::Done => true,
            MarketRequestState::Processing {
                signals_active,
                signals_finished,
                ..
            } => signals_active.is_empty() && signals_finished.is_empty(),
        }
    }

    fn is_stale(&self, timeout: Duration) -> bool {
        match self {
            MarketRequestState::Done => false,
            MarketRequestState::Processing { last_update, .. } => last_update.elapsed() >= timeout,
        }
    }

    fn time_to_stale(&self, timeout: Duration) -> Duration {
        match self {
            MarketRequestState::Done => timeout,
            MarketRequestState::Processing { last_update, .. } => {
                // Don't want negative values
                timeout - last_update.elapsed().min(timeout)
            }
        }
    }

    fn retain_fresh_signals(&mut self) {
        if let MarketRequestState::Processing {
            signals_active,
            signals_finished,
            ..
        } = self
        {
            signals_active.retain(|signal| !is_signal_done(signal));
            signals_finished.retain(|signal| !is_signal_done(signal));
        }
    }

    async fn wait_for_update(&self, max_timeout: Duration) -> bool {
        let stale = self.time_to_stale(max_timeout);
        let timeout = stale.max(DUR_MIN_WAIT);

        let start = Instant::now();
        while start.elapsed() < timeout {
            let needs_update = self.are_any_signals_done();
            if needs_update {
                return true;
            }
            sleep(DUR_MIN_WAIT).await;
        }

        self.is_stale(max_timeout)
    }

    async fn process_handle(&mut self, socket: &mut WebSocket) -> Result<()> {
        let MarketRequestState::Processing {
            handle,
            last_update,
            ..
        } = self
        else {
            return Ok(());
        };

        *last_update = Instant::now();

        let output = if let Some(result) = handle.now_or_never() {
            let ProcessorHandleOutput {
                listings,
                history,
                failure_ids,
            } = result?;
            *self = MarketRequestState::Done;
            Output::Result {
                listings,
                history,
                failures: failure_ids,
            }
        } else {
            match handle.status().values() {
                Status::Text(status) => Output::TextStatus { status },
                Status::Processing(values) => Output::DetailedStatus {
                    status: values.into_iter().map(DetailedStatus::from).collect(),
                },
            }
        };
        let message_text = serde_json::to_string(&output)?;
        socket.send(Message::Text(message_text)).await?;

        Ok(())
    }
}
