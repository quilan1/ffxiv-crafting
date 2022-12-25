#![allow(clippy::unused_async)]

use std::{collections::BTreeMap, sync::Arc};

use axum::{
    extract::{Form, Json, State},
    response::IntoResponse,
};
use futures::{future::BoxFuture, FutureExt};
use log::info;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    server::server::ServerState,
    universalis::{ItemListing, MarketItemInfoMap, UniversalisProcessor, UniversalisStatus},
    util::ProcessType,
};

use super::{make_builder, not_found, ok_json};

////////////////////////////////////////////////////////////
// Stuff that gets sent in/out from the server

#[derive(Deserialize, Debug)]
pub struct PutInput {
    pub filters: String,
    pub data_center: Option<String>,
    pub retain_num_days: Option<f32>,
}

#[derive(Deserialize, Debug)]
pub struct GetInput {
    id: String,
}

#[derive(Serialize, Debug)]
struct Output {
    id: String,
    status: String,
    output_info: Option<OutputInfo>,
}

#[derive(Serialize, Debug)]
pub struct OutputInfo {
    pub item_info: BTreeMap<u32, CustomItemInfo>,
    pub top_ids: Vec<u32>,
}

#[derive(Serialize, Debug)]
pub struct CustomItemInfo {
    item_id: u32,
    name: String,
    listings: Vec<ItemListing>,
    history: Vec<ItemListing>,
    recipe: Option<Recipe>,
}

#[derive(Serialize, Debug)]
struct Recipe {
    inputs: Vec<RecipeData>,
    outputs: u32,
}

#[derive(Serialize, Debug)]
struct RecipeData {
    item_id: u32,
    count: u32,
}

////////////////////////////////////////////////////////////

pub struct CustomStateInfo {
    pub status: UniversalisStatus,
    pub output: BoxFuture<'static, MarketItemInfoMap>,
    pub top_ids: Vec<u32>,
}

////////////////////////////////////////////////////////////

pub struct Custom;

impl Custom {
    // Check if a previous queued item request is done. If not, return the current progress.
    pub async fn get_lazy(
        State(state): State<Arc<ServerState>>,
        Form(payload): Form<GetInput>,
    ) -> impl IntoResponse {
        enum CurrentStatus {
            Error(String),
            InProgress(String),
            Finished(MarketItemInfoMap),
        }

        info!("[get_lazy] Payload {payload:?}");

        let uuid = payload.id;
        let current_status = state.with_custom(&uuid, |info| match info {
            None => CurrentStatus::Error(format!("Id not found: {uuid}")),
            Some(info) => match (&mut info.output).now_or_never() {
                Some(result) => CurrentStatus::Finished(result),
                None => CurrentStatus::InProgress(info.status.to_string()),
            },
        });

        match current_status {
            CurrentStatus::Error(err) => not_found(err).into_response(),
            CurrentStatus::InProgress(status) => {
                ok_json(Output::from_in_progress(uuid, status)).into_response()
            }
            CurrentStatus::Finished(mb_info_map) => {
                let info = state.remove_custom(&uuid).unwrap();
                let top_ids = info.top_ids;
                let out = Output::from_finished(uuid, util::json_results(top_ids, mb_info_map));
                ok_json(out).into_response()
            }
        }
    }

    // Queue up a future, and create a future_output variable in which to store the result
    pub async fn put_lazy(
        State(state): State<Arc<ServerState>>,
        Json(payload): Json<PutInput>,
    ) -> impl IntoResponse {
        info!("[put_lazy] Payload {payload:?}");

        let (top_ids, all_ids) = util::get_ids_from_filters(payload.filters);
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
        state.insert_custom(
            &uuid,
            CustomStateInfo {
                status: status.clone(),
                output,
                top_ids,
            },
        );

        ok_json(Output::from_in_progress(uuid, status))
    }
}

////////////////////////////////////////////////////////////

impl Output {
    fn from_in_progress<S: AsRef<str>, T: std::fmt::Display>(uuid: S, status: T) -> Self {
        Self {
            id: uuid.as_ref().into(),
            status: status.to_string(),
            output_info: None,
        }
    }

    fn from_finished<S: AsRef<str>>(uuid: S, output: OutputInfo) -> Self {
        Output {
            id: uuid.as_ref().into(),
            status: "Finished...".into(),
            output_info: Some(output),
        }
    }
}

////////////////////////////////////////////////////////////

impl ServerState {
    fn insert_custom<S: AsRef<str>>(&self, uuid: S, info: CustomStateInfo) {
        let mut records = self.lazy_records.lock();
        records.entry(uuid.as_ref().into()).or_insert(info);
    }

    fn remove_custom<S: AsRef<str>>(&self, uuid: S) -> Option<CustomStateInfo> {
        let mut records = self.lazy_records.lock();
        records.remove(uuid.as_ref())
    }

    fn with_custom<S: AsRef<str>, F, T>(&self, uuid: S, func: F) -> T
    where
        F: Fn(Option<&mut CustomStateInfo>) -> T,
    {
        let mut records = self.lazy_records.lock();
        func(records.get_mut(uuid.as_ref()))
    }
}

////////////////////////////////////////////////////////////

mod util {
    use std::collections::{BTreeMap, BTreeSet};

    use crate::{
        library::{library, Filter},
        universalis::{MarketItemInfo, MarketItemInfoMap},
        util::item_name,
    };

    use super::{CustomItemInfo, OutputInfo, Recipe, RecipeData};

    pub fn get_ids_from_filters<S: AsRef<str>>(filters: S) -> (Vec<u32>, Vec<u32>) {
        fn push_ids(ids: &mut Vec<u32>, item_id: u32) {
            ids.push(item_id);
            if !library().all_recipes.contains_item_id(item_id) {
                return;
            }

            for input in &library().all_recipes[&item_id].inputs {
                push_ids(ids, input.item_id);
            }
        }

        let filters = filters.as_ref();
        let item_list = library().all_items.items.values().collect::<Vec<_>>();
        let (items, _) = Filter::apply_filters(item_list, filters);

        let ids = items
            .iter()
            .flat_map(|item| {
                let mut item_ids = Vec::new();
                push_ids(&mut item_ids, item.id);
                item_ids
            })
            // .filter(|item_id| !item(item_id).is_untradable)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();

        let items = items.into_iter().map(|item| item.id).collect::<Vec<_>>();

        (items, ids)
    }

    pub fn json_results(top_ids: Vec<u32>, mb_info: MarketItemInfoMap) -> OutputInfo {
        let mut out_items = BTreeMap::new();
        for (id, MarketItemInfo { listings, history }) in mb_info {
            out_items.insert(
                id,
                CustomItemInfo {
                    item_id: id,
                    name: item_name(&id).replace('\u{00A0}', " ").to_string(),
                    listings,
                    history,
                    recipe: recipe_info(id),
                },
            );
        }

        for &id in &top_ids {
            if out_items.contains_key(&id) {
                continue;
            }

            out_items.insert(
                id,
                CustomItemInfo {
                    item_id: id,
                    name: item_name(&id).to_string(),
                    listings: Vec::new(),
                    history: Vec::new(),
                    recipe: recipe_info(id),
                },
            );
        }

        OutputInfo {
            top_ids,
            item_info: out_items,
        }
    }

    fn recipe_info(id: u32) -> Option<Recipe> {
        library().all_recipes.get(id).map(|recipe| Recipe {
            outputs: recipe.output.count,
            inputs: recipe
                .inputs
                .iter()
                .map(|input| RecipeData {
                    item_id: input.item_id,
                    count: input.count,
                })
                .collect(),
        })
    }
}
