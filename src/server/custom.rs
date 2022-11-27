use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
    time::Duration,
};

use axum::{
    extract::{Form, Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use axum_macros::debug_handler;
use futures::FutureExt;
use log::info;
use serde::{Deserialize, Serialize};
use tokio::time::sleep;

use crate::{
    library::{
        AnalysisFilters, Filter, Ingredient, RecursiveMarketBoardAnalysis, VelocityAnalysis,
        WorldInfo,
    },
    new_universalis::{
        ItemListing, MarketItemInfo, MarketItemInfoMap, UniversalisBuilder, UniversalisProcessor,
    },
    server::server::ServerState,
    // universalis::{Universalis, AsyncProcessor, UniversalisProcessor, UniversalisBuilder, UniversalisData},
    util::{item, item_name, library},
};

#[derive(Deserialize, Debug)]
pub struct CustomFilter {
    filters: String,
    data_centers: Option<String>,
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

#[derive(Serialize, Debug)]
struct ItemInfo {
    item_id: u32,
    name: String,
    listings: Vec<ItemListing>,
    history: Vec<ItemListing>,
    recipe: Option<Recipe>,
}

#[derive(Serialize, Debug)]
pub struct JsonFilter {
    name: String,
    values: Vec<String>,
}

#[derive(Serialize, Debug)]
pub struct CustomOut {
    filters: Vec<JsonFilter>,
    item_info: BTreeMap<u32, ItemInfo>,
    top_ids: Vec<u32>,
}

pub struct Custom;

impl Custom {
    // #[debug_handler]
    pub async fn custom_filter(
        State(state): State<Arc<ServerState<'_>>>,
        Form(payload): Form<CustomFilter>,
    ) -> impl IntoResponse {
        info!("GET custom_filter: Payload {payload:?}");

        let (top_ids, ids, filters) = get_ids_from_filters(payload.filters);

        let builder = {
            let builder = UniversalisBuilder::new();
            match payload.data_centers {
                None => builder,
                Some(data_centers) => {
                    builder.data_centers(data_centers.split(",").collect::<Vec<_>>())
                }
            }
        };

        // println!("ids: {ids:?}, top_ids: {top_ids:?}");
        let mb_info_map =
            UniversalisProcessor::process_ids(state.processor.clone(), &builder, ids.clone()).await;

        let out = json_results(top_ids, filters, mb_info_map);

        (StatusCode::OK, Json(out))
    }
}

fn get_ids_from_filters(filters: String) -> (Vec<u32>, Vec<u32>, Vec<Filter>) {
    let item_list = library().all_items.items.values().collect::<Vec<_>>();
    let (items, filters) = Filter::apply_filters(item_list, &filters);

    fn push_ids(ids: &mut Vec<u32>, item_id: u32) {
        ids.push(item_id);
        if !library().all_recipes.contains_item_id(item_id) {
            return;
        }

        for input in &library().all_recipes[&item_id].inputs {
            push_ids(ids, input.item_id);
        }
    }

    let ids = items
        .iter()
        .map(|item| {
            let mut item_ids = Vec::new();
            push_ids(&mut item_ids, item.id);
            item_ids
        })
        .flatten()
        // .filter(|item_id| !item(item_id).is_untradable)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    let items = items.into_iter().map(|item| item.id).collect::<Vec<_>>();

    (items, ids, filters)
}

fn json_results(top_ids: Vec<u32>, filters: Vec<Filter>, mb_info: MarketItemInfoMap) -> CustomOut {
    let mut out_items = BTreeMap::new();
    for (id, MarketItemInfo { listings, history }) in mb_info {
        out_items.insert(
            id,
            ItemInfo {
                item_id: id,
                name: item_name(id).to_string(),
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
            ItemInfo {
                item_id: id,
                name: item_name(id).to_string(),
                listings: Vec::new(),
                history: Vec::new(),
                recipe: recipe_info(id),
            },
        );
    }

    CustomOut {
        filters: filters.into_iter().map(|filter| filter.into()).collect(),
        top_ids,
        item_info: out_items,
    }
}

fn recipe_info(id: u32) -> Option<Recipe> {
    if let Some(recipe) = library().all_recipes.get(&id) {
        Some(Recipe {
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
    } else {
        None
    }
}

impl From<Filter> for JsonFilter {
    fn from(filter: Filter) -> Self {
        Self {
            name: filter.ftype,
            values: filter.options,
        }
    }
}
