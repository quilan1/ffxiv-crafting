use std::{collections::BTreeMap, sync::Arc};

use axum::{
    extract::{Form, Json, State},
    response::{IntoResponse, Response},
};
use futures::{future::BoxFuture, FutureExt};
use log::info;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    server::{
        custom_util::{get_ids_from_filters, json_results},
        server::ServerState,
    },
    universalis::{MarketItemInfoMap, UniversalisProcessor, UniversalisStatus},
    util::ProcessType,
};

use super::{custom_util::CustomItemInfo, make_builder, not_found, ok_json};

#[derive(Deserialize, Debug)]
pub struct CustomInput {
    pub filters: String,
    pub data_center: Option<String>,
    pub retain_num_days: Option<f32>,
}

#[derive(Deserialize, Debug)]
pub struct CustomLazyInput {
    id: String,
}

#[derive(Serialize, Debug)]
pub struct CustomOut {
    pub item_info: BTreeMap<u32, CustomItemInfo>,
    pub top_ids: Vec<u32>,
}

#[derive(Serialize, Debug)]
struct CustomLazyOutput {
    id: String,
    status: String,
    output: Option<CustomOut>,
}

pub struct CustomLazyInfo {
    pub status: UniversalisStatus,
    pub output: BoxFuture<'static, MarketItemInfoMap>,
    pub top_ids: Vec<u32>,
}

enum CurrentStatus {
    Error(String),
    InProgress(String),
    Finished(MarketItemInfoMap),
}

pub struct Custom;

impl Custom {
    // Check if a previous queued item request is done. If not, return the current progress.
    pub async fn get_lazy(
        State(state): State<Arc<ServerState>>,
        Form(payload): Form<CustomLazyInput>,
    ) -> impl IntoResponse {
        info!("[get_lazy] Payload {payload:?}");

        let uuid = payload.id;
        let current_status = state.with_lazy(&uuid, |info| match info {
            None => CurrentStatus::Error(format!("Id not found: {uuid}")),
            Some(info) => {
                match (&mut info.output).now_or_never() {
                    Some(result) => CurrentStatus::Finished(result),
                    None => CurrentStatus::InProgress(info.status.to_string()),
                }
            }
        });

        match current_status {
            CurrentStatus::Error(err) => not_found(err).into_response(),
            CurrentStatus::InProgress(status) => ok_json(CustomLazyOutput::from_in_progress(uuid, status)).into_response(),
            CurrentStatus::Finished(mb_info_map) => {
                let info = state.remove_lazy(&uuid).unwrap();
                let top_ids = info.top_ids;
                let out = CustomLazyOutput::from_finished(uuid, json_results(top_ids, mb_info_map));
                ok_json(out).into_response()
            }
        }
    }

    // Queue up a future, and create a future_output variable in which to store the result
    pub async fn put_lazy(
        State(state): State<Arc<ServerState>>,
        Json(payload): Json<CustomInput>,
    ) -> impl IntoResponse {
        info!("[put_lazy] Payload {payload:?}");

        let (top_ids, all_ids) = get_ids_from_filters(payload.filters);
        let builder = make_builder(payload.data_center);

        // Queue up the future
        let status = UniversalisStatus::new();
        let future = UniversalisProcessor::process_ids(
            state.async_processor.clone(),
            builder.data_centers,
            all_ids,
            payload.retain_num_days.unwrap_or(7.0),
            status.clone(),
        )
        .boxed();

        // Send it off for processing, via the unlimited queue
        let output = state
            .async_processor
            .clone()
            .process_future(future, ProcessType::Unlimited);

        // Save the placeholder & output into the server state
        let uuid = Uuid::new_v4().to_string();
        state.insert_lazy(
            &uuid,
            CustomLazyInfo {
                status: status.clone(),
                output,
                top_ids,
            },
        );

        ok_json(CustomLazyOutput::from_in_progress(uuid, status))
    }
}

impl CustomLazyOutput {
    fn from_in_progress<S: AsRef<str>, T: std::fmt::Display>(uuid: S, status: T) -> Self {
        Self {
            id: uuid.as_ref().into(),
            status: status.to_string(),
            output: None,
        }
    }

    fn from_finished<S: AsRef<str>>(uuid: S, output: CustomOut) -> Self {
        CustomLazyOutput {
            id: uuid.as_ref().into(),
            status: "Finished...".into(),
            output: Some(output),
        }
    }
}

impl ServerState {
    fn insert_lazy<S: AsRef<str>>(&self, uuid: S, info: CustomLazyInfo) {
        let mut records = self.lazy_records.lock();
        records.entry(uuid.as_ref().into()).or_insert(info);
    }

    fn remove_lazy<S: AsRef<str>>(&self, uuid: S) -> Option<CustomLazyInfo> {
        let mut records = self.lazy_records.lock();
        records.remove(uuid.as_ref())
    }

    fn with_lazy<S: AsRef<str>, F, T>(&self, uuid: S, func: F) -> T
    where
        F: Fn(Option<&mut CustomLazyInfo>) -> T,
    {
        let mut records = self.lazy_records.lock();
        func(records.get_mut(uuid.as_ref()))
    }
}
