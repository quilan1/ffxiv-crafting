use std::sync::Arc;

use axum::{extract::State, response::IntoResponse, Form, Json};
use axum_macros::debug_handler;
use ffxiv_items::get_ids_from_filters;
use ffxiv_universalis::{
    GenListing, History, ItemListingMap, Listing, ProcessType, UniversalisProcessor,
    UniversalisStatus,
};
use futures::{future::BoxFuture, FutureExt};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::server::ServerState;
use crate::util::{make_builder, not_found, ok_json, ok_text};

#[derive(Deserialize)]
pub struct PutInput {
    filters: String,
    data_center: Option<String>,
    retain_num_days: Option<f32>,
}

#[derive(Deserialize, Debug)]
pub struct GetInput {
    id: String,
}

#[derive(Serialize)]
struct GetOutput {
    id: String,
    status: String,
    output_info: Option<GetOutputInfo>,
}

#[derive(Serialize)]
struct GetOutputInfo {
    pub listings: ItemListingMap,
    pub failures: Vec<u32>,
}

pub enum GetStatus {
    Error(String),
    InProgress(String),
    Finished(ItemListingMap, Vec<u32>),
}

pub struct ListingInfo {
    pub status: UniversalisStatus,
    pub output: BoxFuture<'static, (ItemListingMap, Vec<u32>)>,
    pub top_ids: Vec<u32>,
    pub retain_num_days: f32,
}

////////////////////////////////////////////////////////////

#[debug_handler]
#[allow(clippy::unused_async)]
pub async fn put_item_history(
    State(state): State<Arc<ServerState>>,
    Json(payload): Json<PutInput>,
) -> impl IntoResponse {
    ok_text(put_item_gen_listing_data::<History>(&state, payload))
}

#[debug_handler]
#[allow(clippy::unused_async)]
pub async fn put_item_listings(
    State(state): State<Arc<ServerState>>,
    Json(payload): Json<PutInput>,
) -> impl IntoResponse {
    ok_text(put_item_gen_listing_data::<Listing>(&state, payload))
}

#[debug_handler]
pub async fn get_item_history(
    State(state): State<Arc<ServerState>>,
    Form(payload): Form<GetInput>,
) -> impl IntoResponse {
    get_gen_listings(state, payload).await
}

#[debug_handler]
pub async fn get_item_listings(
    State(state): State<Arc<ServerState>>,
    Form(payload): Form<GetInput>,
) -> impl IntoResponse {
    get_gen_listings(state, payload).await
}

#[allow(clippy::unused_async)]
pub async fn get_gen_listings(state: Arc<ServerState>, payload: GetInput) -> impl IntoResponse {
    let uuid = payload.id.clone();
    let current_status = get_item_gen_listing_data(&state.clone(), payload);

    match current_status {
        GetStatus::Error(err) => not_found(err).into_response(),
        GetStatus::InProgress(status) => {
            ok_json(GetOutput::from_in_progress(uuid, status)).into_response()
        }
        GetStatus::Finished(listing_map, failures) => {
            state.remove_listing(&uuid).unwrap();
            let out = GetOutput::from_finished(uuid, GetOutputInfo::new(listing_map, failures));
            ok_json(out).into_response()
        }
    }
}

////////////////////////////////////////////////////////////

pub fn put_item_gen_listing_data<T: GenListing + 'static>(
    state: &Arc<ServerState>,
    payload: PutInput,
) -> String {
    let (top_ids, all_ids) = get_ids_from_filters(payload.filters);
    let builder = make_builder(payload.data_center);

    let processor =
        UniversalisProcessor::new(state.async_processor.clone(), builder.data_centers, all_ids);

    // Queue up the future
    let status = UniversalisStatus::default();
    let future = processor.process_listings::<T>(status.clone()).boxed();

    // Send it off for processing, via the unlimited queue
    let output = state
        .async_processor
        .clone()
        .process_future(future, ProcessType::Unlimited);

    let retain_num_days = payload.retain_num_days.unwrap_or(7.0);

    // Save the placeholder & output into the server state
    let uuid = Uuid::new_v4().to_string();
    state.insert_listing(
        &uuid,
        ListingInfo {
            status: status.clone(),
            output,
            top_ids,
            retain_num_days,
        },
    );

    uuid
}

pub fn get_item_gen_listing_data(state: &Arc<ServerState>, payload: GetInput) -> GetStatus {
    let uuid = payload.id;
    state.with_listing(&uuid, |info| match info {
        None => GetStatus::Error(format!("Id not found: {uuid}")),
        Some(info) => match (&mut info.output).now_or_never() {
            Some((info, failures)) => GetStatus::Finished(info, failures),
            None => GetStatus::InProgress(info.status.to_string()),
        },
    })
}

////////////////////////////////////////////////////////////

impl GetOutput {
    fn from_in_progress<S: AsRef<str>, T: std::fmt::Display>(uuid: S, status: T) -> Self {
        Self {
            id: uuid.as_ref().into(),
            status: status.to_string(),
            output_info: None,
        }
    }

    fn from_finished<S: AsRef<str>>(uuid: S, output: GetOutputInfo) -> Self {
        Self {
            id: uuid.as_ref().into(),
            status: "Finished...".into(),
            output_info: Some(output),
        }
    }
}

////////////////////////////////////////////////////////////

impl GetOutputInfo {
    fn new(listings: ItemListingMap, failures: Vec<u32>) -> Self {
        Self { listings, failures }
    }
}

////////////////////////////////////////////////////////////

impl ServerState {
    fn insert_listing<S: AsRef<str>>(&self, uuid: S, info: ListingInfo) {
        let mut listings = self.listings.lock();
        listings.entry(uuid.as_ref().into()).or_insert(info);
    }

    fn remove_listing<S: AsRef<str>>(&self, uuid: S) -> Option<ListingInfo> {
        let mut listings = self.listings.lock();
        listings.remove(uuid.as_ref())
    }

    fn with_listing<S: AsRef<str>, F, T>(&self, uuid: S, func: F) -> T
    where
        F: Fn(Option<&mut ListingInfo>) -> T,
    {
        let mut listings = self.listings.lock();
        func(listings.get_mut(uuid.as_ref()))
    }
}
