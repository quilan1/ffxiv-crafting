use std::time::{Duration, Instant};

use anyhow::Result;
use axum::extract::ws::{Message, WebSocket};
use ffxiv_universalis::{
    ReqwestDownloader, Signal, UniversalisHandle, UniversalisHistory, UniversalisListing,
    UniversalisProcessor, UniversalisStatusValues,
};
use futures::{Future, FutureExt};
use tokio::time::sleep;

use super::{DetailedStatus, Input, Output};

////////////////////////////////////////////////////////////

const DUR_MIN_WAIT: Duration = Duration::from_millis(10);
const DUR_TIMEOUT: Duration = Duration::from_millis(10000);

pub async fn wait_for_universalis(
    socket: &mut WebSocket,
    universalis_processor: &UniversalisProcessor,
    payload: Input,
    all_ids: &[u32],
    server_uuid: &str,
) -> Result<()> {
    let mut market_request_info =
        make_market_request_info(universalis_processor, payload, all_ids, server_uuid).await;

    // Send over initial messages
    market_request_info
        .process_handles(socket, (true, true))
        .await?;
    market_request_info.retain_fresh_signals();

    // Wait until we've finished all history and listings
    let timeouts = [DUR_TIMEOUT, Duration::from_millis(50)];
    while !market_request_info.is_done() {
        if let Some(None) = socket.recv().now_or_never() {
            log::info!(target: "ffxiv_server", "{server_uuid} WebSocket closed unexpectedly");
            break;
        }

        let timeout = timeouts[usize::from(market_request_info.is_waiting_for_cleanup())];
        let update = market_request_info.wait_for_update(timeout).await;
        market_request_info.process_handles(socket, update).await?;
        market_request_info.retain_fresh_signals();
    }
    Ok(())
}

async fn make_market_request_info(
    universalis_processor: &UniversalisProcessor,
    payload: Input,
    all_ids: &[u32],
    server_uuid: &str,
) -> MarketRequestInfo {
    let worlds: Vec<_> = payload
        .data_center
        .or(std::env::var("FFXIV_DATA_CENTERS").ok())
        .unwrap()
        .split(',')
        .map(str::trim)
        .map(ToString::to_string)
        .collect();

    let retain_num_days = payload.retain_num_days.unwrap_or(7.0);

    let history_handle = universalis_processor
        .make_request::<UniversalisHistory, ReqwestDownloader>(&worlds, all_ids, retain_num_days);

    let listing_handle = universalis_processor
        .make_request::<UniversalisListing, ReqwestDownloader>(&worlds, all_ids, retain_num_days);

    log::info!(target: "ffxiv_server",
        "Server uuid {server_uuid} maps to history universalis uuid {}",
        history_handle.uuid()
    );
    log::info!(target: "ffxiv_server",
        "Server uuid {server_uuid} maps to listing universalis uuid {}",
        listing_handle.uuid()
    );

    let history_signals = history_handle.status().signals().await;
    let listing_signals = listing_handle.status().signals().await;
    MarketRequestInfo::new(
        history_handle,
        listing_handle,
        history_signals,
        listing_signals,
    )
}

////////////////////////////////////////////////////////////

enum MarketRequestState {
    Processing {
        handle: UniversalisHandle,
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
        handle: UniversalisHandle,
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

    async fn process_handle(&mut self, socket: &mut WebSocket, listing_type: &str) -> Result<()> {
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
            let (listings, failures) = result?;
            *self = MarketRequestState::Done;
            Output::Result {
                listing_type: listing_type.into(),
                listings,
                failures,
            }
        } else {
            match handle.status().values() {
                UniversalisStatusValues::Text(status) => Output::TextStatus {
                    listing_type: listing_type.into(),
                    status,
                },
                UniversalisStatusValues::Processing(values) => Output::DetailedStatus {
                    listing_type: listing_type.into(),
                    status: values.into_iter().map(DetailedStatus::from).collect(),
                },
            }
        };
        let message_text = serde_json::to_string(&output)?;
        socket.send(Message::Text(message_text)).await?;

        Ok(())
    }
}

////////////////////////////////////////////////////////////

struct MarketRequestInfo {
    history: MarketRequestState,
    listing: MarketRequestState,
}

impl MarketRequestInfo {
    fn new(
        history_handle: UniversalisHandle,
        listing_handle: UniversalisHandle,
        history_signals: (Vec<Signal<()>>, Vec<Signal<bool>>),
        listing_signals: (Vec<Signal<()>>, Vec<Signal<bool>>),
    ) -> Self {
        Self {
            history: MarketRequestState::new(history_handle, history_signals),
            listing: MarketRequestState::new(listing_handle, listing_signals),
        }
    }

    fn are_any_signals_done(&self) -> bool {
        self.history.are_any_signals_done() || self.listing.are_any_signals_done()
    }

    fn is_done(&self) -> bool {
        self.history.is_done() && self.listing.is_done()
    }

    fn is_waiting_for_cleanup(&self) -> bool {
        self.history.is_waiting_for_cleanup() && self.listing.is_waiting_for_cleanup()
    }

    fn retain_fresh_signals(&mut self) {
        self.history.retain_fresh_signals();
        self.listing.retain_fresh_signals();
    }

    async fn wait_for_update(&self, max_timeout: Duration) -> (bool, bool) {
        let stale_history = self.history.time_to_stale(max_timeout);
        let stale_listing = self.listing.time_to_stale(max_timeout);
        let timeout = stale_history.min(stale_listing).max(DUR_MIN_WAIT);

        let start = Instant::now();
        while start.elapsed() < timeout {
            let needs_update = self.are_any_signals_done();
            if needs_update {
                return (true, true);
            }
            sleep(DUR_MIN_WAIT).await;
        }

        (
            self.history.is_stale(max_timeout),
            self.listing.is_stale(max_timeout),
        )
    }

    async fn process_handles(
        &mut self,
        socket: &mut WebSocket,
        (history_update, listing_update): (bool, bool),
    ) -> Result<()> {
        if history_update {
            self.history.process_handle(socket, "history").await?;
        }
        if listing_update {
            self.listing.process_handle(socket, "listing").await?;
        }
        Ok(())
    }
}
