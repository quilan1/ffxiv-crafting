#[macro_use]
mod table;

mod filter;
mod ingredient_table;
mod input_ids;
mod item_db;
mod item_db_filter;
mod item_db_items;
mod item_info_table;
mod recipe_table;
mod ui_category_table;

use filter::Filter;
use ingredient_table::IngredientTable;
use input_ids::InputIdsTable;
use item_info_table::ItemInfoTable;
use recipe_table::RecipeTable;
use ui_category_table::UiCategoryTable;

pub use item_db::ItemDB;

pub(super) const BIND_MAX: usize = 65535;
