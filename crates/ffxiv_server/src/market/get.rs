use std::sync::Arc;

use axum::extract::Path;
use axum::{extract::State, response::IntoResponse};
use ffxiv_universalis::ItemMarketInfoMap;
use futures::FutureExt;
use serde::Serialize;
use tokio::task::spawn_blocking;

use crate::util::{not_found, ok_json};

use super::MarketState;

////////////////////////////////////////////////////////////

#[derive(Serialize)]
struct GetOutput {
    status: String,
    output_info: Option<GetOutputInfo>,
}

#[derive(Serialize)]
struct GetOutputInfo {
    listings: ItemMarketInfoMap,
    failures: Vec<u32>,
}

pub enum GetStatus {
    Error(String),
    InProgress(String),
    Finished(ItemMarketInfoMap, Vec<u32>),
}

////////////////////////////////////////////////////////////

pub async fn get_market_info(
    State(state): State<Arc<MarketState>>,
    Path(uuid): Path<String>,
) -> impl IntoResponse {
    spawn_blocking(move || get_market_request_status(&state, &uuid))
        .await
        .unwrap()
}

////////////////////////////////////////////////////////////

pub fn get_market_request_status(state: &Arc<MarketState>, uuid: &str) -> impl IntoResponse {
    match get_market_request_data(state, uuid) {
        GetStatus::Error(err) => not_found(err).into_response(),
        GetStatus::InProgress(status) => {
            ok_json(GetOutput::from_in_progress(status)).into_response()
        }
        GetStatus::Finished(listing_map, failures) => {
            state.remove_handle(uuid).unwrap();
            let out = GetOutput::from_finished(GetOutputInfo::new(listing_map, failures));
            ok_json(out).into_response()
        }
    }
}

pub fn get_market_request_data(state: &Arc<MarketState>, uuid: &str) -> GetStatus {
    state.with_handle(uuid, |info| match info {
        None => GetStatus::Error(format!("Id not found: {uuid}")),
        Some(universalis_handle) => match universalis_handle.now_or_never() {
            Some(Ok((info, failures))) => GetStatus::Finished(info, failures),
            Some(Err(err)) => GetStatus::Error(err.to_string()),
            None => GetStatus::InProgress(universalis_handle.status().text()),
        },
    })
}

impl GetOutput {
    fn from_in_progress<T: std::fmt::Display>(status: T) -> Self {
        Self {
            status: status.to_string(),
            output_info: None,
        }
    }

    fn from_finished(output: GetOutputInfo) -> Self {
        Self {
            status: "Finished...".into(),
            output_info: Some(output),
        }
    }
}

////////////////////////////////////////////////////////////

impl GetOutputInfo {
    fn new(listings: ItemMarketInfoMap, failures: Vec<u32>) -> Self {
        Self { listings, failures }
    }
}

////////////////////////////////////////////////////////////
