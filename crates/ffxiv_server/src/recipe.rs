use std::{collections::BTreeMap, sync::Arc};

use axum::{
    extract::{Form, State},
    response::IntoResponse,
};
use ffxiv_items::Library;
use serde::{Deserialize, Serialize};
use tokio::task::spawn_blocking;

use crate::util::ok_json;

#[derive(Deserialize)]
pub struct GetInput {
    pub filters: String,
}

#[derive(Serialize)]
pub struct GetOutput {
    top_ids: Vec<u32>,
    item_info: BTreeMap<u32, ItemInfo>,
}

#[derive(Serialize)]
pub struct ItemInfo {
    item_id: u32,
    name: String,
    recipe: Option<Recipe>,
}

#[derive(Serialize)]
pub struct Recipe {
    pub inputs: Vec<RecipeData>,
    pub outputs: u32,
}

#[derive(Serialize)]
pub struct RecipeData {
    pub item_id: u32,
    pub count: u32,
}

////////////////////////////////////////////////////////////

// Return recipe info for a particular filter
pub async fn get_recipe_info(
    State(library): State<Arc<Library>>,
    Form(payload): Form<GetInput>,
) -> impl IntoResponse {
    ok_json(
        spawn_blocking(move || get_recipe_info_data(&library, payload))
            .await
            .unwrap(),
    )
}

fn get_recipe_info_data(library: &Library, payload: GetInput) -> GetOutput {
    let (top_ids, all_ids) = ffxiv_items::get_ids_from_filters(library, payload.filters);
    let item_info = all_ids
        .into_iter()
        .map(|id| {
            (
                id,
                ItemInfo {
                    item_id: id,
                    name: library.item_name(&id).replace('\u{00A0}', " ").to_string(),
                    recipe: recipe_info(library, id),
                },
            )
        })
        .collect();

    GetOutput { top_ids, item_info }
}

////////////////////////////////////////////////////////////

fn recipe_info(library: &Library, id: u32) -> Option<Recipe> {
    library
        .item_info_checked(&id)
        .and_then(|item| item.recipe.as_ref())
        .map(|recipe| Recipe {
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
