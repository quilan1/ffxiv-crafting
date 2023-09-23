use std::{collections::BTreeMap, sync::Arc};

use anyhow::Result;
use axum::{
    extract::{Form, State},
    response::IntoResponse,
};
use ffxiv_items::ItemDB;
use serde::{Deserialize, Serialize};
use tokio::task::spawn_blocking;

use crate::{JsonResponse, StringResponse};

////////////////////////////////////////////////////////////

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
    pub inputs: Vec<Ingredient>,
    pub outputs: u32,
}

#[derive(Serialize)]
pub struct Ingredient {
    pub item_id: u32,
    pub count: u32,
}

////////////////////////////////////////////////////////////

// Return recipe info for a particular filter
pub async fn get_recipe_info(
    State(db): State<Arc<ItemDB>>,
    Form(payload): Form<GetInput>,
) -> impl IntoResponse {
    log::info!(target: "ffxiv_server", "Fetching recipe for '{}'", payload.filters);
    match get_recipe_info_data(&db, payload).await {
        Ok(v) => v.ok(),
        Err(e) => e.to_string().server_error(),
    }
}

async fn get_recipe_info_data(db: &ItemDB, payload: GetInput) -> Result<GetOutput> {
    let (top_ids, all_ids) = db.get_ids_from_filters(&payload.filters).await?;
    let items = db.items_from_ids(&all_ids).await?;

    let item_info = spawn_blocking(|| {
        items
            .into_iter()
            .map(|item| {
                (
                    item.id,
                    ItemInfo {
                        item_id: item.id,
                        name: item.name,
                        recipe: item.recipe.map(Into::into),
                    },
                )
            })
            .collect()
    })
    .await?;

    Ok(GetOutput { top_ids, item_info })
}

////////////////////////////////////////////////////////////

impl From<ffxiv_items::Recipe> for Recipe {
    fn from(recipe: ffxiv_items::Recipe) -> Self {
        Self {
            inputs: recipe.inputs.into_iter().map(Into::into).collect(),
            outputs: recipe.output.count,
        }
    }
}

impl From<ffxiv_items::Ingredient> for Ingredient {
    fn from(ingredient: ffxiv_items::Ingredient) -> Self {
        Self {
            item_id: ingredient.item_id,
            count: ingredient.count,
        }
    }
}
