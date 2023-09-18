use anyhow::Result;

use crate::{ItemMarketInfoMap, UniversalisJson};

pub struct UniversalisListing;
pub struct UniversalisHistory;

pub trait UniversalisRequestType {
    fn url<S: AsRef<str>>(world: S, ids: S) -> String;
    fn fetch_type() -> &'static str;
    fn parse_json(json: String, retain_num_days: f32) -> Result<ItemMarketInfoMap>;
}

impl UniversalisRequestType for UniversalisListing {
    fn url<S: AsRef<str>>(world: S, ids: S) -> String {
        let universalis_url = universalis_url();
        format!(
            "{universalis_url}/api/v2/{}/{}?entries=0",
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

impl UniversalisRequestType for UniversalisHistory {
    fn url<S: AsRef<str>>(world: S, ids: S) -> String {
        let universalis_url = universalis_url();
        format!(
            "{universalis_url}/api/v2/history/{}/{}",
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

fn universalis_url() -> String {
    std::env::var("UNIVERSALIS_URL")
        .ok()
        .unwrap_or("https://universalis.app".into())
}
