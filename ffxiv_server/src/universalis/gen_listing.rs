use anyhow::Result;

use super::{json::UniversalisJson, ItemListingMap};

pub struct Listing;
pub struct History;

pub trait GenListing {
    fn url<S: AsRef<str>>(world: S, ids: S) -> String;
    fn fetch_type() -> &'static str;
    fn parse_json(json: String, retain_num_days: f32) -> Result<ItemListingMap>;
}

impl GenListing for Listing {
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

    fn parse_json(json: String, retain_num_days: f32) -> Result<ItemListingMap> {
        UniversalisJson::parse_listing(json, retain_num_days)
    }
}

impl GenListing for History {
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

    fn parse_json(json: String, retain_num_days: f32) -> Result<ItemListingMap> {
        UniversalisJson::parse_history(json, retain_num_days)
    }
}
