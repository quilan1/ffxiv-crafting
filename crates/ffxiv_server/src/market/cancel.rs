use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use tokio::task::spawn_blocking;

use crate::util::{not_found, ok_text};

use super::MarketState;

pub async fn put_market_cancel(
    State(state): State<Arc<MarketState>>,
    Path(uuid): Path<String>,
) -> impl IntoResponse {
    fn inner(state: &Arc<MarketState>, uuid: String) -> impl IntoResponse {
        let status = state.with_handle(&uuid, |info| {
            info.map(|universalis_handle| universalis_handle.status())
        });

        match status {
            Some(_) => {
                log::info!("[Cancel] {uuid} market call cancelled");
                state.remove_handle(uuid);
                ok_text("OK").into_response()
            }
            None => not_found(format!("UUID {uuid} not found!")).into_response(),
        }
    }

    spawn_blocking(move || inner(&state, uuid)).await.unwrap()
}
