use std::io::Cursor;

use anyhow::Result;
use futures::join;
use reqwest::IntoUrl;

pub struct CsvContent {
    pub item: Cursor<String>,
    pub item_ui_category: Cursor<String>,

    pub class_job_category: Cursor<String>,
    pub craft_leve: Cursor<String>,
    pub leve: Cursor<String>,

    pub recipe: Cursor<String>,
    pub recipe_level: Cursor<String>,
}

impl CsvContent {
    pub async fn download() -> Result<Self> {
        let (
            item,
            item_ui_category,
            class_job_category,
            craft_leve,
            leve,
            recipe,
            recipe_level_table,
        ) = join!(
            Self::download_file("Item.csv"),
            Self::download_file("ItemUICategory.csv"),
            Self::download_file("ClassJobCategory.csv"),
            Self::download_file("CraftLeve.csv"),
            Self::download_file("Leve.csv"),
            Self::download_file("Recipe.csv"),
            Self::download_file("RecipeLevelTable.csv"),
        );

        let (
            item,
            item_ui_category,
            class_job_category,
            craft_leve,
            leve,
            recipe,
            recipe_level_table,
        ) = (
            Cursor::new(item?),
            Cursor::new(item_ui_category?),
            Cursor::new(class_job_category?),
            Cursor::new(craft_leve?),
            Cursor::new(leve?),
            Cursor::new(recipe?),
            Cursor::new(recipe_level_table?),
        );

        Ok(Self {
            item,
            item_ui_category,
            class_job_category,
            craft_leve,
            leve,
            recipe,
            recipe_level: recipe_level_table,
        })
    }

    async fn download_file(file_name: &str) -> Result<String> {
        Self::download_url(format!(
            "https://raw.githubusercontent.com/xivapi/ffxiv-datamining/master/csv/{file_name}"
        ))
        .await
    }

    async fn download_url<S: IntoUrl>(url: S) -> Result<String> {
        Ok(reqwest::get(url).await?.text().await?)
    }
}
