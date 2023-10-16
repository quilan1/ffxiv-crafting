#[macro_use]
mod table;

mod ingredient_table;
mod input_ids_table;
mod item_info_table;
mod recipe_table;
mod ui_category_table;
mod update_table;

pub use ingredient_table::IngredientTable;
pub use input_ids_table::InputIdsTable;
pub use item_info_table::{ItemInfoTable, ItemInfoTableBuilder};
pub use recipe_table::{RecipeTable, RecipeTableBuilder};
pub use ui_category_table::{UiCategoryTable, UiCategoryTableBuilder};
pub use update_table::UpdateTable;

pub(super) const BIND_MAX: usize = 65535;

pub fn strip_whitespace<S: AsRef<str>>(s: S) -> String {
    use regex::Regex;
    let re = Regex::new(r"\s+").unwrap();
    re.replace_all(s.as_ref(), " ").into()
}

async fn download_csv<F: mock_traits::FileDownloader>(file_name: &str) -> anyhow::Result<String> {
    F::download(&format!(
        "https://raw.githubusercontent.com/xivapi/ffxiv-datamining/master/csv/{file_name}"
    ))
    .await
}
