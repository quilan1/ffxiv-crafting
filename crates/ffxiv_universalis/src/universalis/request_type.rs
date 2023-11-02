use anyhow::Result;

use super::{ListingsMap, RequestResult, UniversalisJson};

////////////////////////////////////////////////////////////

#[derive(Clone)]
pub enum RequestType {
    Listing,
    History,
}

////////////////////////////////////////////////////////////

impl RequestType {
    pub fn url<S: AsRef<str>>(&self, world: S, ids: S) -> String {
        match self {
            RequestType::Listing => format!(
                "https://universalis.app/api/v2/{}/{}?entries=0",
                world.as_ref(),
                ids.as_ref()
            ),
            RequestType::History => format!(
                "https://universalis.app/api/v2/history/{}/{}?entriesWithin={}",
                world.as_ref(),
                ids.as_ref(),
                2 * 7 * 24 * 60 * 60, // two weeks, in seconds
            ),
        }
    }

    pub fn fetch_type(&self) -> &'static str {
        match self {
            RequestType::Listing => "listing",
            RequestType::History => "history",
        }
    }

    pub fn parse_json(&self, json: String, retain_num_days: f32) -> Result<ListingsMap> {
        match self {
            RequestType::Listing => UniversalisJson::parse_listing(json, retain_num_days),
            RequestType::History => UniversalisJson::parse_history(json, retain_num_days),
        }
    }

    pub fn result_listings(&self, listings: ListingsMap) -> RequestResult {
        match self {
            RequestType::Listing => RequestResult::Listing(listings),
            RequestType::History => RequestResult::History(listings),
        }
    }
}
