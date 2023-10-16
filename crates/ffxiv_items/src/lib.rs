#![warn(unused_crate_dependencies)]

#[macro_use]
mod csv_parse;

mod filter;
mod github_metadata;
mod item_db;
mod item_db_filter;
mod item_db_items;
mod item_id;
mod item_info;
mod parsers;
mod recipe;
mod tables;

use filter::{Filter, FilterBindingInfo};
use github_metadata::last_updated_from_github;
use item_id::ItemId;

pub use item_db::ItemDB;
pub use item_info::ItemInfo;
pub use recipe::{Ingredient, Recipe};

mod _temp {
    use chrono as _;
    use env_logger as _;
    use tokio as _;
}
