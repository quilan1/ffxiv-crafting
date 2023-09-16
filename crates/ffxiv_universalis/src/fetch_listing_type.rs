use anyhow::Result;

use crate::{ItemMarketInfoMap, UniversalisJson};

pub struct Listing;
pub struct History;

pub trait MarketRequestType {
    fn url<S: AsRef<str>>(world: S, ids: S) -> String;
    fn fetch_type() -> &'static str;
    fn parse_json(json: String, retain_num_days: f32) -> Result<ItemMarketInfoMap>;
}

impl MarketRequestType for Listing {
    fn url<S: AsRef<str>>(world: S, ids: S) -> String {
        format!(
            "https://universalis.app/api/v2/{}/{}?entries=0",
            world.as_ref(),
            ids.as_ref()
        )
    }

    fn fetch_type() -> &'static str {
        "listing"
    }

    fn parse_json(json: String, retain_num_days: f32) -> Result<ItemMarketInfoMap> {
        UniversalisJson::parse_listing(json, retain_num_days)
    }
}

impl MarketRequestType for History {
    fn url<S: AsRef<str>>(world: S, ids: S) -> String {
        format!(
            "https://universalis.app/api/v2/history/{}/{}",
            world.as_ref(),
            ids.as_ref()
        )
    }

    fn fetch_type() -> &'static str {
        "history"
    }

    fn parse_json(json: String, retain_num_days: f32) -> Result<ItemMarketInfoMap> {
        UniversalisJson::parse_history(json, retain_num_days)
    }
}
