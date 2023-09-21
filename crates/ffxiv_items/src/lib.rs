#![warn(unused_crate_dependencies)]

mod csv_content;
mod db;
mod filter;
mod item_id;
mod item_info;
mod library;
mod parsers;
mod recipe;

#[cfg(test)]
mod library_test_data;

use csv_content::CsvContent;
use filter::Filter;
use item_id::ItemId;
use recipe::{Ingredient, Recipe, RecipeLevelInfo};

pub use db::ItemDB;
pub use item_id::get_ids_from_filters;
pub use item_info::ItemInfo;
pub use library::Library;

#[cfg(test)]
mod tests {
    use tokio as _;
}
