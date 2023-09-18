#![warn(unused_crate_dependencies)]
mod handle;
mod json;
mod json_types;
mod processor;
mod processor_data;
mod request;
mod request_type;
mod status;

use json::{ItemListing, UniversalisJson};
use processor::MAX_UNIVERSALIS_CONCURRENT_FUTURES;
use processor_data::UniversalisProcessorData;
use request::UniversalisRequest;
use status::UniversalisStatusState;

pub use handle::{UniversalisHandle, UniversalisHandleOutput};
pub use json::ItemMarketInfoMap;
pub use processor::{new_universalis_processor, request_universalis_info};
pub use request_type::{UniversalisHistory, UniversalisListing, UniversalisRequestType};
pub use status::UniversalisStatus;

////////////////////////////////////////////////////////////
