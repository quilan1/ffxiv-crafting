use std::collections::BTreeMap;

use ffxiv_universalis::{ItemMarketInfoMap, UniversalisProcessorState};
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
#[serde(rename_all = "camelCase")]
pub enum Output {
    Recipe {
        top_ids: Vec<u32>,
        item_info: BTreeMap<u32, ItemInfo>,
    },
    Result {
        listing_type: String,
        listings: ItemMarketInfoMap,
        failures: Vec<u32>,
    },
    TextStatus {
        listing_type: String,
        status: String,
    },
    DetailedStatus {
        listing_type: String,
        status: Vec<DetailedStatus>,
    },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DetailedStatus {
    Active,
    Finished(bool),
    Queued(i32),
}

impl From<UniversalisProcessorState> for DetailedStatus {
    fn from(value: UniversalisProcessorState) -> Self {
        match value {
            UniversalisProcessorState::Active => Self::Active,
            UniversalisProcessorState::Finished(successful) => Self::Finished(successful),
            UniversalisProcessorState::Queued(position) => Self::Queued(position),
        }
    }
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
