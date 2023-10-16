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
pub use item_info_table::ItemInfoTable;
pub use recipe_table::RecipeTable;
pub use ui_category_table::UiCategoryTable;
pub use update_table::UpdateTable;

pub(super) const BIND_MAX: usize = 65535;

async fn download_file(file_name: &str) -> anyhow::Result<String> {
    download_url(format!(
        "https://raw.githubusercontent.com/xivapi/ffxiv-datamining/master/csv/{file_name}"
    ))
    .await
}

pub async fn download_commits(file_name: &str) -> anyhow::Result<String> {
    use reqwest::header::{HeaderMap, HeaderValue};
    let url = format!(
        "https://api.github.com/repos/xivapi/ffxiv-datamining/commits?path=csv/{file_name}&page=1&page_per=1"
    );

    let mut headers = HeaderMap::new();
    headers.insert("User-Agent", HeaderValue::from_static("Tiny Rust Program"));
    let client = reqwest::ClientBuilder::new()
        .default_headers(headers)
        .build()?;
    let request = client.get(url).build()?;
    Ok(client.execute(request).await?.text().await?)
}

async fn download_url<S: reqwest::IntoUrl>(url: S) -> anyhow::Result<String> {
    Ok(reqwest::get(url).await?.text().await?)
}

pub fn strip_whitespace<S: AsRef<str>>(s: S) -> String {
    use regex::Regex;
    let re = Regex::new(r"\s+").unwrap();
    re.replace_all(s.as_ref(), " ").into()
}
