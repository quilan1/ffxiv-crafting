use std::{collections::BTreeMap, sync::Arc};

use anyhow::Result;
use axum::{
    extract::{Form, State},
    response::IntoResponse,
};
use ffxiv_items::ItemDB;
use serde::{Deserialize, Serialize};

use crate::{JsonResponse, StringResponse};

#[derive(Deserialize)]
pub struct GetInput {
    pub filters: String,
}

#[derive(Serialize)]
pub struct GetOutput {
    top_ids: Vec<u32>,
    item_info: BTreeMap<u32, ItemInfo>,
}
impl JsonResponse for GetOutput {}

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
    State(db): State<Arc<ItemDB>>,
    Form(payload): Form<GetInput>,
) -> impl IntoResponse {
    match get_recipe_info_data(&db, payload).await {
        Ok(v) => v.ok(),
        Err(e) => e.to_string().server_error(),
    }
}

async fn get_recipe_info_data(db: &ItemDB, payload: GetInput) -> Result<GetOutput> {
    let (top_ids, all_ids) = db.get_ids_from_filters(&payload.filters).await?;
    let items = db.items_from_ids(&all_ids).await?;

    let item_info = items
        .into_iter()
        .map(|item| {
            (
                item.id,
                ItemInfo {
                    item_id: item.id,
                    name: item.name,
                    recipe: item.recipe.map(|recipe| Recipe {
                        inputs: recipe
                            .inputs
                            .into_iter()
                            .map(|input| RecipeData {
                                item_id: input.item_id,
                                count: input.count,
                            })
                            .collect(),
                        outputs: recipe.output.count,
                    }),
                },
            )
        })
        .collect();

    Ok(GetOutput { top_ids, item_info })
}
