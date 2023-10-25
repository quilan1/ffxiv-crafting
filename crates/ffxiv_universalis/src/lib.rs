#![warn(unused_crate_dependencies)]
#![allow(clippy::module_inception)]
#![warn(missing_docs)]

//! This crate takes in numeric item IDs from `ffxiv_items` and produces a
//! stream that one can continuously poll to retrieve the listing & history
//! information for the item IDs.
//!
//! # Example
//!
//! ```rust,no_run
//! use ffxiv_items::ItemDB;
//! use ffxiv_universalis::{ListingsResults, PacketResult, Processor};
//! use futures::StreamExt;
//! use mock_traits::{ReqwestDownloader};
//!
//! const ITEM_DB_CONN: &str = "mysql://<user>:<password>@<server>:<port>/<database>";
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Initialize the item database
//!     let db = ItemDB::connect(ITEM_DB_CONN).await?;
//!     db.initialize::<ReqwestDownloader>().await?;
//!
//!     // Prepare the values to request from universalis
//!     let retain_num_days = 7.0;
//!     let worlds = [String::from("Seraph")];
//!     let ids = db
//!         .ids_from_query(":rlevel 90, :cat !Metal|Lumber|Leather|Stone|Cloth|Reagent")
//!         .await?;
//!
//!     // Request listings for the item ids
//!     let processor = Processor::new();
//!     let mut request =
//!         processor.make_request::<ReqwestDownloader>(&worlds, &ids, retain_num_days);
//!
//!     // Spawn the server
//!     let server = tokio::spawn(processor.async_processor());
//!
//!     // Return the results
//!     while let Some(packet_result) = request.next().await {
//!         match packet_result {
//!             PacketResult::Success(listings, history) => {
//!                 println!(
//!                     "Success: {} listing IDs, {} history IDs",
//!                     listings.len(),
//!                     history.len()
//!                 );
//!             }
//!             PacketResult::Failure(ids) => {
//!                 println!("Failure: {ids:?}");
//!             }
//!         }
//!     }
//!
//!     // Alternatively one can use:
//!     let ListingsResults { listings, history, failures } = request.collect_all().await;
//!
//!     // Stop the server
//!     processor.async_processor().disconnect();
//!     server.await?;
//!
//!     Ok(())
//! }
//! ```

mod multi_signal;
mod processor;
mod universalis;

////////////////////////////////////////////////////////////

use multi_signal::{multi_signal, MSender};

pub use multi_signal::MReceiver;
pub use processor::{ListingsResults, PacketResult, Processor, ProcessorHandle, Status};
use universalis::AsyncProcessorHandle;
pub use universalis::{AsyncProcessor, ItemListing, ListingsMap, RequestState};

#[doc(hidden)]
pub mod json {
    pub use crate::universalis::json_types::*;
}

#[cfg(test)]
mod tests {
    use ffxiv_items as _;
}
