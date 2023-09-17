use std::sync::Arc;

use axum::{extract::State, response::IntoResponse, Json};
use ffxiv_items::get_ids_from_filters;
use ffxiv_universalis::{History, Listing, MarketRequestType, UniversalisProcessor};
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

pub async fn put_market_history(
    State(state): State<Arc<MarketState>>,
    Json(payload): Json<PutInput>,
) -> impl IntoResponse {
    let uuid = spawn_blocking(move || put_market_request::<History>(&state, payload))
        .await
        .unwrap();
    ok_text(uuid)
}

pub async fn put_market_listings(
    State(state): State<Arc<MarketState>>,
    Json(payload): Json<PutInput>,
) -> impl IntoResponse {
    let uuid = spawn_blocking(move || put_market_request::<Listing>(&state, payload))
        .await
        .unwrap();
    ok_text(uuid)
}

////////////////////////////////////////////////////////////

pub fn put_market_request<T: MarketRequestType + 'static>(
    state: &Arc<MarketState>,
    payload: PutInput,
) -> String {
    let (_, all_ids) = get_ids_from_filters(payload.filters);
    let worlds = payload
        .data_center
        .or(std::env::var("FFXIV_DATA_CENTERS").ok())
        .unwrap()
        .split(',')
        .map(ToString::to_string)
        .collect();

    // Send the request over to the async processor
    let retain_num_days = payload.retain_num_days.unwrap_or(7.0);
    let universalis_handle = UniversalisProcessor::market_info::<T>(
        state.async_processor.clone(),
        worlds,
        all_ids,
        retain_num_days,
    );

    // Save the placeholder & output into the server state
    let uuid = Uuid::new_v4().to_string();
    state.insert_market_request(&uuid, universalis_handle);

    uuid
}
