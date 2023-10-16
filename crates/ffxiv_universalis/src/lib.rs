#![warn(unused_crate_dependencies)]
mod handle;
mod json;
pub mod json_types;
mod processor;
mod processor_data;
mod request;
mod request_type;
mod status;

use json::{ItemListing, UniversalisJson};
use processor::MAX_UNIVERSALIS_CONCURRENT_FUTURES;
use processor_data::UniversalisProcessorData;
use request::{UniversalisRequest, UniversalisRequestHandle};
use status::UniversalisStatusState;

pub use handle::{UniversalisHandle, UniversalisHandleOutput};
pub use json::ItemMarketInfoMap;
pub use processor::UniversalisProcessor;
pub use request::Signal;
pub use request_type::{UniversalisHistory, UniversalisListing, UniversalisRequestType};
pub use status::{UniversalisProcessorState, UniversalisStatus, UniversalisStatusValues};

////////////////////////////////////////////////////////////
