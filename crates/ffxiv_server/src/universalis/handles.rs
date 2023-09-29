use anyhow::Result;
use axum::extract::ws::{Message, WebSocket};
use ffxiv_universalis::{
    UniversalisHandle, UniversalisHistory, UniversalisListing, UniversalisProcessor,
};
use futures::FutureExt;

use super::{Input, ListingOutput, ListingStatus};

////////////////////////////////////////////////////////////

pub fn make_universalis_handles(
    universalis_processor: &UniversalisProcessor,
    payload: Input,
    all_ids: Vec<u32>,
    server_uuid: &str,
) -> (Option<UniversalisHandle>, Option<UniversalisHandle>) {
    let worlds: Vec<_> = payload
        .data_center
        .or(std::env::var("FFXIV_DATA_CENTERS").ok())
        .unwrap()
        .split(',')
        .map(str::trim)
        .map(ToString::to_string)
        .collect();

    let retain_num_days = payload.retain_num_days.unwrap_or(7.0);

    let history_handle = universalis_processor.make_request::<UniversalisHistory>(
        worlds.clone(),
        all_ids.clone(),
        retain_num_days,
    );

    let listing_handle =
        universalis_processor.make_request::<UniversalisListing>(worlds, all_ids, retain_num_days);

    log::info!(target: "ffxiv_server",
        "Server uuid {server_uuid} maps to history universalis uuid {}",
        history_handle.uuid()
    );
    log::info!(target: "ffxiv_server",
        "Server uuid {server_uuid} maps to listing universalis uuid {}",
        listing_handle.uuid()
    );

    (Some(history_handle), Some(listing_handle))
}

pub async fn process_universalis_handle(
    socket: &mut WebSocket,
    listing_type: &str,
    universalis_handle: &mut Option<UniversalisHandle>,
) -> Result<()> {
    let Some(handle) = universalis_handle.as_mut() else {
        return Ok(());
    };

    let message_text = if let Some(result) = handle.now_or_never() {
        let (listings, failures) = result?;
        universalis_handle.take();
        serde_json::to_string(&ListingOutput {
            msg_type: "output".into(),
            listing_type: listing_type.into(),
            listings,
            failures,
        })?
    } else {
        serde_json::to_string(&ListingStatus {
            msg_type: "status".into(),
            listing_type: listing_type.into(),
            status: handle.status().text(),
        })?
    };
    socket.send(Message::Text(message_text)).await?;

    Ok(())
}
