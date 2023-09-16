use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use ffxiv_universalis::UniversalisStatusValue;
use tokio::spawn;

use crate::util::{not_found, ok_text};

use super::MarketState;

pub async fn put_market_cancel(
    State(state): State<Arc<MarketState>>,
    Path(uuid): Path<String>,
) -> impl IntoResponse {
    fn inner(state: &Arc<MarketState>, uuid: String) -> impl IntoResponse {
        let status = state.with_market_request(&uuid, |info| match info {
            Some(market_info) => Some(market_info.status.clone()),
            None => None,
        });

        let status = match status {
            None => return not_found(format!("UUID {uuid} not found!")).into_response(),
            Some(s) => s,
        };

        let UniversalisStatusValue::Processing(ids) = status.value() else {
            return ok_text("OK").into_response();
        };

        let id_count = ids.len();
        state.async_processor.cancel(ids);
        log::info!("[Cancel] {uuid} market call ended ({id_count} fetch requests)");

        state.remove_market_request(uuid);
        ok_text("OK").into_response()
    }

    spawn(async move { inner(&state, uuid) }).await.unwrap()
}
