use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use ffxiv_items::Library;
use tokio::task::spawn_blocking;

use crate::StringResponse;

use super::MarketState;

pub async fn put_market_cancel(
    State((state, _library)): State<(Arc<MarketState>, Arc<Library>)>,
    Path(uuid): Path<String>,
) -> impl IntoResponse {
    fn inner(state: &Arc<MarketState>, uuid: String) -> impl IntoResponse {
        let status = state.with_handle(&uuid, |info| {
            info.map(|universalis_handle| universalis_handle.status())
        });

        match status {
            Some(_) => {
                log::info!(target: "ffxiv_server", "{uuid} market call cancelled");
                state.remove_handle(uuid);
                "OK".ok()
            }
            None => format!("UUID {uuid} not found!").not_found(),
        }
    }

    spawn_blocking(move || inner(&state, uuid)).await.unwrap()
}
