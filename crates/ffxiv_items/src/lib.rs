#![warn(unused_crate_dependencies)]

mod csv_content;
mod db;
mod item_id;
mod item_info;
mod parsers;
mod recipe;

use csv_content::CsvContent;
use item_id::ItemId;
use item_info::ItemDBInfo;
use recipe::RecipeLevelInfo;

pub use db::ItemDB;
pub use item_info::ItemInfo;
pub use recipe::{Ingredient, Recipe};

#[cfg(test)]
mod tests {
    use tokio as _;
}
