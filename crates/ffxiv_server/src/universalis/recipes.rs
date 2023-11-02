use anyhow::Result;
use axum::extract::ws::WebSocket;
use tokio::task::spawn_blocking;

use super::{write_message, Ingredient, ItemInfo, Output, Recipe};

////////////////////////////////////////////////////////////

pub async fn send_recipes(
    socket: &mut WebSocket,
    top_ids: &[u32],
    items: Vec<ffxiv_items::ItemInfo>,
    is_compressed: bool,
) -> Result<()> {
    let recipe_text = get_recipe_info_data(top_ids, items).await?;
    write_message(socket, recipe_text, is_compressed).await?;
    Ok(())
}

////////////////////////////////////////////////////////////

async fn get_recipe_info_data(
    top_ids: &[u32],
    items: Vec<ffxiv_items::ItemInfo>,
) -> Result<String> {
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

    Ok(serde_json::to_string(&Output::Recipe {
        top_ids: top_ids.to_vec(),
        item_info,
    })?)
}

////////////////////////////////////////////////////////////

impl From<ffxiv_items::Recipe> for Recipe {
    fn from(recipe: ffxiv_items::Recipe) -> Self {
        Self {
            inputs: recipe.inputs.into_iter().map(Into::into).collect(),
            outputs: recipe.output.count,
            level: recipe.level,
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
