#[macro_use]
mod table;

mod ingredient_table;
mod input_ids_table;
mod item_info_table;
mod recipe_table;
mod ui_category_table;

pub use ingredient_table::IngredientTable;
pub use input_ids_table::InputIdsTable;
pub use item_info_table::ItemInfoTable;
pub use recipe_table::RecipeTable;
pub use ui_category_table::UiCategoryTable;

pub(super) const BIND_MAX: usize = 65535;

async fn download_file(file_name: &str) -> anyhow::Result<String> {
    download_url(format!(
        "https://raw.githubusercontent.com/xivapi/ffxiv-datamining/master/csv/{file_name}"
    ))
    .await
}

async fn download_url<S: reqwest::IntoUrl>(url: S) -> anyhow::Result<String> {
    Ok(reqwest::get(url).await?.text().await?)
}

pub fn strip_whitespace<S: AsRef<str>>(s: S) -> String {
    use regex::Regex;
    let re = Regex::new(r"\s+").unwrap();
    re.replace_all(s.as_ref(), " ").into()
}
