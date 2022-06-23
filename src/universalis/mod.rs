mod json;
mod processor;
mod universalis;

use json::process_json;
use processor::Processor;
use universalis::UniversalisRequest;
pub use universalis::{ItemListing, MarketBoardInfo, MarketBoardItemInfo, Universalis};
