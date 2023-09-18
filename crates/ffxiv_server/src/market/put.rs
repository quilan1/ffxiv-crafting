use std::sync::Arc;

use axum::{extract::State, response::IntoResponse, Json};
use ffxiv_items::{get_ids_from_filters, Library};
use ffxiv_universalis::{
    request_universalis_info, UniversalisHistory, UniversalisListing, UniversalisRequestType,
};
use log::info;
use serde::Deserialize;
use tokio::task::spawn_blocking;
use uuid::Uuid;

use crate::util::ok_text;

use super::MarketState;

////////////////////////////////////////////////////////////

#[derive(Deserialize)]
pub struct PutInput {
    filters: String,
    data_center: Option<String>,
    retain_num_days: Option<f32>,
}

////////////////////////////////////////////////////////////

#[allow(clippy::unused_async)]
pub async fn put_market_history(
    State((state, library)): State<(Arc<MarketState>, Arc<Library>)>,
    Json(payload): Json<PutInput>,
) -> impl IntoResponse {
    let uuid = Uuid::new_v4().to_string();
    let uuid_clone = uuid.clone();
    spawn_blocking(move || {
        put_market_request::<UniversalisHistory>(&state, &library, uuid_clone, payload)
    });
    ok_text(uuid)
}

#[allow(clippy::unused_async)]
pub async fn put_market_listings(
    State((state, library)): State<(Arc<MarketState>, Arc<Library>)>,
    Json(payload): Json<PutInput>,
) -> impl IntoResponse {
    let uuid = Uuid::new_v4().to_string();
    let uuid_clone = uuid.clone();
    spawn_blocking(move || {
        put_market_request::<UniversalisListing>(&state, &library, uuid_clone, payload)
    });
    ok_text(uuid)
}

////////////////////////////////////////////////////////////

pub fn put_market_request<T: UniversalisRequestType>(
    state: &Arc<MarketState>,
    library: &Library,
    uuid: String,
    payload: PutInput,
) -> String {
    let (_, all_ids) = get_ids_from_filters(library, payload.filters);
    let worlds = payload
        .data_center
        .or(std::env::var("FFXIV_DATA_CENTERS").ok())
        .unwrap()
        .split(',')
        .map(str::trim)
        .map(ToString::to_string)
        .collect();

    // Send the request over to the async processor
    let retain_num_days = payload.retain_num_days.unwrap_or(7.0);
    let universalis_handle = request_universalis_info::<T>(
        state.async_processor.clone(),
        worlds,
        all_ids,
        retain_num_days,
    );

    info!(
        "[Server] Server uuid {uuid} maps to universalis uuid {}",
        universalis_handle.uuid()
    );

    // Save the placeholder & output into the server state
    state.insert_handle(&uuid, universalis_handle);

    uuid
}
