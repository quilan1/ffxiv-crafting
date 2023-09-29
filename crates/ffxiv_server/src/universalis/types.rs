use std::collections::BTreeMap;

use ffxiv_universalis::ItemMarketInfoMap;
use serde::{Deserialize, Serialize};

////////////////////////////////////////////////////////////

#[derive(Deserialize)]
pub struct Input {
    pub filters: String,
    pub data_center: Option<String>,
    pub retain_num_days: Option<f32>,
}

////////////////////////////////////////////////////////////

#[derive(Serialize)]
pub struct RecipeOutput {
    pub msg_type: String,
    pub top_ids: Vec<u32>,
    pub item_info: BTreeMap<u32, ItemInfo>,
}

#[derive(Serialize)]
pub struct ListingOutput {
    pub msg_type: String,
    pub listing_type: String,
    pub listings: ItemMarketInfoMap,
    pub failures: Vec<u32>,
}

#[derive(Serialize)]
pub struct ListingStatus {
    pub msg_type: String,
    pub listing_type: String,
    pub status: String,
}

////////////////////////////////////////////////////////////

#[derive(Serialize)]
pub struct ItemInfo {
    pub item_id: u32,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recipe: Option<Recipe>,
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
