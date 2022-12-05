use std::{collections::BTreeMap, sync::Arc};

use axum::{
    extract::{Form, Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use log::info;
use serde::{Deserialize, Serialize};

use crate::{
    server::{
        custom_util::{get_ids_from_filters, json_results},
        server::ServerState,
    },
    universalis::{UniversalisBuilder, UniversalisProcessor},
};

use super::custom_util::CustomItemInfo;

#[derive(Deserialize, Debug)]
pub struct CustomFilter {
    filters: String,
    data_center: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct CustomOut {
    pub item_info: BTreeMap<u32, CustomItemInfo>,
    pub top_ids: Vec<u32>,
}

pub struct Custom;

impl Custom {
    // #[debug_handler]
    pub async fn custom_filter(
        State(state): State<Arc<ServerState>>,
        Form(payload): Form<CustomFilter>,
    ) -> impl IntoResponse {
        info!("GET custom_filter: Payload {payload:?}");

        let (top_ids, all_ids) = get_ids_from_filters(payload.filters);

        let builder = {
            let builder = UniversalisBuilder::new();
            match payload.data_center {
                None => builder,
                Some(data_centers) => {
                    builder.data_centers(data_centers.split(",").collect::<Vec<_>>())
                }
            }
        };

        let mb_info_map =
            UniversalisProcessor::process_ids(state.async_processor.clone(), &builder, all_ids)
                .await;

        let out = json_results(top_ids, mb_info_map);

        (StatusCode::OK, Json(out))
    }
}
