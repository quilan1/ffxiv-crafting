use std::collections::BTreeMap;

use ffxiv_universalis::{ListingsMap, RequestState};
use serde::{Deserialize, Serialize};

////////////////////////////////////////////////////////////

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Input {
    pub query: String,
    pub data_center: Option<String>,
    pub retain_num_days: Option<f32>,
}

////////////////////////////////////////////////////////////

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Output {
    #[serde(rename_all = "camelCase")]
    Recipe {
        top_ids: Vec<u32>,
        item_info: BTreeMap<u32, ItemInfo>,
    },
    #[serde(rename_all = "camelCase")]
    Success {
        listings: ListingsMap,
        history: ListingsMap,
    },
    #[serde(rename_all = "camelCase")]
    Failure { failures: Vec<u32> },
    #[serde(rename_all = "camelCase")]
    DetailedStatus { status: Vec<DetailedStatus> },
    #[serde(rename_all = "camelCase")]
    Done {},
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DetailedStatus {
    Active,
    Warn,
    Finished(bool),
    Queued(i32),
}

impl From<RequestState> for DetailedStatus {
    fn from(value: RequestState) -> Self {
        match value {
            RequestState::Active => Self::Active,
            RequestState::Warn => Self::Warn,
            RequestState::Finished(successful) => Self::Finished(successful),
            RequestState::Queued(position) => Self::Queued(position),
        }
    }
}

////////////////////////////////////////////////////////////

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
pub struct Ingredient {
    pub item_id: u32,
    pub count: u32,
}
