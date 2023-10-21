#![warn(unused_crate_dependencies)]
#![warn(missing_docs)]

//! This crate is used to perform queries on FFXIV items. Many of the exported
//! functions and features involve returning item ids and recipe information
//! for items associated with a particular search query.
//!
//! To use the library, one would start with initializing an [ItemDB] object like such:
//! ```rust
//! use ffxiv_items::ItemDB;
//! use mock_traits::ReqwestDownloader;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn Error>> {
//!     // ITEM_DB_CONN will look like, e.g:
//!     // "mysql://<user>:<password>@<server>:<port>/<database>"
//!     let db = ItemDB::connect(ITEM_DB_CONN).await?;
//!     db.initialize<ReqwestDownloader>().await?;
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
