mod json;
mod processor;
mod universalis;

use json::process_json;
use processor::ProcessorStream;
use universalis::UniversalisRequest;
pub use universalis::{ItemListing, MarketBoardInfo, MarketBoardItemInfo, Universalis};
