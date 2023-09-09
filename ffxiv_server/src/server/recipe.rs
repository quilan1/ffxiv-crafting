use std::collections::BTreeMap;

use axum::{extract::Form, response::IntoResponse};
use axum_macros::debug_handler;
use serde::{Deserialize, Serialize};
use ffxiv_items::{library, item_name};

use super::{ok_json, util};

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
#[allow(clippy::unused_async)]
#[debug_handler]
pub async fn get_recipe_info(Form(payload): Form<GetInput>) -> impl IntoResponse {
    ok_json(get_recipe_info_data(payload))
}

pub fn get_recipe_info_data(payload: GetInput) -> GetOutput {
    let (top_ids, all_ids) = util::get_ids_from_filters(payload.filters);
    let item_info = all_ids
        .into_iter()
        .map(|id| {
            (
                id,
                ItemInfo {
                    item_id: id,
                    name: item_name(&id).replace('\u{00A0}', " ").to_string(),
                    recipe: recipe_info(id),
                },
            )
        })
        .collect();

    GetOutput { top_ids, item_info }
}

////////////////////////////////////////////////////////////

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
