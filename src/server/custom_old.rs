use std::sync::Arc;

use axum::{extract::State, response::IntoResponse, Form};
use log::info;

use crate::universalis::{UniversalisProcessor, UniversalisStatus};

use super::{
    custom::{Custom, CustomInput},
    custom_util::{get_ids_from_filters, json_results},
    make_builder, ok_json,
    server::ServerState,
};

impl Custom {
    // #[debug_handler]
    pub async fn custom_filter(
        State(state): State<Arc<ServerState>>,
        Form(payload): Form<CustomInput>,
    ) -> impl IntoResponse {
        info!("GET custom_filter: Payload {payload:?}");

        let (top_ids, all_ids) = get_ids_from_filters(payload.filters);
        let builder = make_builder(payload.data_center);

        let mb_info_map = UniversalisProcessor::process_ids(
            state.async_processor.clone(),
            builder.data_centers.clone(),
            all_ids,
            UniversalisStatus::new(),
        )
        .await;

        ok_json(json_results(top_ids, mb_info_map))
    }
}
