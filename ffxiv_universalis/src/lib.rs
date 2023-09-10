mod builder;
mod gen_listing;
mod json;
mod processor;
mod status;
mod util;

pub use builder::UniversalisBuilder;
pub use gen_listing::{GenListing, History, Listing};
pub use json::ItemListingMap;
pub use processor::UniversalisProcessor;
pub use status::UniversalisStatus;
pub use util::{AmValue, AsyncProcessor, ProcessType};

////////////////////////////////////////////////////////////

// Directly exported as json
#[derive(Debug, Default, serde::Serialize)]
pub struct ItemListing {
    pub price: u32,
    pub count: u32,
    pub is_hq: bool,
    pub world: String,
    pub name: String,
    pub posting: u64,
}
