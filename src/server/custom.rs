use std::{collections::BTreeMap, sync::Arc};

use axum::{
    extract::{Form, Json, State},
    response::{IntoResponse, Response},
};
use futures::FutureExt;
use log::info;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    server::{
        custom_util::{get_ids_from_filters, json_results},
        server::ServerState,
    },
    universalis::{MarketItemInfoMap, UniversalisProcessor, UniversalisStatus},
    util::{AsyncProcessor, SharedFuture},
};

use super::{custom_util::CustomItemInfo, make_builder, not_found, ok_json};

type MarketItemInfoMapOutput = Arc<Mutex<Option<MarketItemInfoMap>>>;

#[derive(Deserialize, Debug)]
pub struct CustomInput {
    pub filters: String,
    pub data_center: Option<String>,
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
    pub future: SharedFuture<()>,
    pub future_output: MarketItemInfoMapOutput,
    pub top_ids: Vec<u32>,
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

        // Check if it's still busy processing
        if let Some(resp) = make_partial_response(&state, &uuid) {
            return resp;
        }

        // We're finished processing, remove it from the records, and send it off
        let info = state.remove_lazy(&uuid).unwrap();

        // Make sure the future's finished up
        info.future.await;

        // Pull the output from the mutex
        let mb_info_map = {
            let mut output = info.future_output.lock();
            std::mem::replace(&mut *output, None).unwrap()
        };

        // Return the final info
        let top_ids = info.top_ids;
        let out = CustomLazyOutput::from_finished(uuid, json_results(top_ids, mb_info_map));
        return ok_json(out).into_response();
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
        let future_output = Arc::new(Mutex::new(None));
        let future = lazy_process_ids(
            state.async_processor.clone(),
            builder.data_centers.clone(),
            all_ids,
            status.clone(),
            future_output.clone(),
        )
        .boxed()
        .shared();

        // Send it off for processing, via the unlimited queue
        state
            .async_general_processor
            .process_lazy(vec![future.clone()]);

        // Save the placeholder & output into the server state
        let uuid = Uuid::new_v4().to_string();
        state.insert_lazy(
            &uuid,
            CustomLazyInfo {
                status: status.clone(),
                future,
                future_output,
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
        F: Fn(Option<&CustomLazyInfo>) -> T,
    {
        let records = self.lazy_records.lock();
        func(records.get(uuid.as_ref()))
    }
}

// Creates a response if the UUID is still being processed, OR if that's an invalid UUID
fn make_partial_response<S: AsRef<str>>(state: &ServerState, uuid: S) -> Option<Response> {
    let uuid = uuid.as_ref();
    state.with_lazy(&uuid, |info| match info {
        None => Some(not_found(format!("Id not found: {}", uuid)).into_response()),
        Some(info) => {
            let status = &info.status;
            if !status.is_finished() {
                let out = CustomLazyOutput::from_in_progress(&uuid, status.to_string());
                Some(ok_json(out).into_response())
            } else {
                None
            }
        }
    })
}

// This is separate, because it's annoying to clone stuff outside of an async move
async fn lazy_process_ids(
    async_processor: AsyncProcessor,
    worlds: Vec<String>,
    all_ids: Vec<u32>,
    status: UniversalisStatus,
    output: MarketItemInfoMapOutput,
) {
    let info = UniversalisProcessor::process_ids(async_processor, worlds, all_ids, status).await;
    output.lock().replace(info);
}
