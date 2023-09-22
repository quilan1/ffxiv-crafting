use std::sync::Arc;

use anyhow::Result;
use axum::{extract::State, response::IntoResponse, Json};
use ffxiv_items::ItemDB;
use ffxiv_universalis::{UniversalisHistory, UniversalisListing, UniversalisRequestType};
use serde::Deserialize;
use uuid::Uuid;

use crate::StringResponse;

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
    State((state, db)): State<(Arc<MarketState>, Arc<ItemDB>)>,
    Json(payload): Json<PutInput>,
) -> impl IntoResponse {
    let uuid = Uuid::new_v4().to_string();
    let uuid_clone = uuid.clone();
    tokio::spawn(async move {
        let _ = put_market_request::<UniversalisHistory>(&state, &db, uuid_clone, payload).await;
    });
    uuid.ok()
}

#[allow(clippy::unused_async)]
pub async fn put_market_listings(
    State((state, db)): State<(Arc<MarketState>, Arc<ItemDB>)>,
    Json(payload): Json<PutInput>,
) -> impl IntoResponse {
    let uuid = Uuid::new_v4().to_string();
    let uuid_clone = uuid.clone();
    tokio::spawn(async move {
        let _ = put_market_request::<UniversalisListing>(&state, &db, uuid_clone, payload).await;
    });
    uuid.ok()
}

////////////////////////////////////////////////////////////

pub async fn put_market_request<T: UniversalisRequestType>(
    state: &Arc<MarketState>,
    db: &ItemDB,
    uuid: String,
    payload: PutInput,
) -> Result<String> {
    let (_, all_ids) = db.get_ids_from_filters(payload.filters).await?;
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
    let universalis_handle = state
        .processor
        .make_request::<T>(worlds, all_ids, retain_num_days);

    log::info!(target: "ffxiv_server",
        "Server uuid {uuid} maps to universalis uuid {}",
        universalis_handle.uuid()
    );

    // Save the placeholder & output into the server state
    state.insert_handle(&uuid, universalis_handle);

    Ok(uuid)
}
