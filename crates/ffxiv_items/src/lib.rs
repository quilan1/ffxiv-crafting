#![warn(unused_crate_dependencies)]
#![warn(missing_docs)]

//! This crate is used to perform queries on FFXIV items. Many of the exported
//! functions and features involve returning item ids and recipe information
//! for items associated with a particular search query.
//!
//! To use the library, one would start with initializing an [ItemDB] object:
//!
//! # Example
//!
//! ```rust,no_run
//! use ffxiv_items::ItemDB;
//! use mock_traits::ReqwestDownloader;
//!
//! const ITEM_DB_CONN: &str = "mysql://<user>:<password>@<server>:<port>/<database>";
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Connect to & initialize the database
//!     let db = ItemDB::connect(ITEM_DB_CONN).await?;
//!     db.initialize::<ReqwestDownloader>().await?;
//!
//!     // Get top-level IDs (items that match the query), associated IDs
//!     // (ingredients of top-level IDs) and item info (id, name & recipe info).
//!     let info = db.all_info_from_query(":name ^Rarefied, :rlevel 61|69").await?;
//!     let (top_level_ids, associated_ids, item_info) = info;
//!
//!     Ok(())
//! }
//! ```

mod csv_parse;
mod github_metadata;
mod item_db;
mod item_db_items;
mod item_db_query;
mod item_id;
mod item_info;
mod parsers;
mod query;
mod recipe;
mod tables;

use csv_parse::csv_parse;
use github_metadata::last_updated_from_github;
use item_id::ItemId;
use query::{Query, QueryBindingInfo};

pub use item_db::ItemDB;
pub use item_info::ItemInfo;
pub use recipe::{Ingredient, Recipe};

mod _temp {
    use chrono as _;
    use env_logger as _;
    use tokio as _;
}
