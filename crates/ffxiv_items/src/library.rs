use anyhow::Result;
use futures::future::join_all;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use std::{fs::DirBuilder, io::Write};

use crate::recipe::RecipeLevelInfo;
use crate::util::ItemId;
use crate::{ItemInfo, Recipe};

use super::parsers::{
    CraftLeveList, ItemList, JobCategoryList, LeveList, RecipeLevelTable, RecipeList,
    UiCategoryList,
};

#[derive(Default)]
pub struct Library {
    pub(crate) all_items: ItemList,
    pub(crate) all_ui_categories: UiCategoryList,
    pub(crate) all_crafting_leves: CraftLeveList,
    pub(crate) all_leves: LeveList,
    pub(crate) all_job_categories: JobCategoryList,
}

impl Library {
    pub async fn create() -> Result<Self> {
        let mut library = Library::default();

        Self::download_files().await?;

        library.all_items = ItemList::from_path("./csv/Item.csv")?;
        library.all_ui_categories = UiCategoryList::from_path("./csv/ItemUICategory.csv")?;

        library.all_crafting_leves = CraftLeveList::from_path("./csv/CraftLeve.csv")?;
        library.all_leves = LeveList::from_path(&library, "./csv/Leve.csv")?;
        library.all_job_categories = JobCategoryList::from_path("./csv/ClassJobCategory.csv")?;

        let recipe_info = Self::parse_recipes()?;
        for (_, recipe) in recipe_info {
            if let Some(item) = library.all_items.items.get_mut(&recipe.output.item_id) {
                item.recipe = Some(recipe);
            }
        }

        library.validate_items();

        Ok(library)
    }

    fn parse_recipes() -> Result<BTreeMap<u32, Recipe>> {
        let all_recipe_levels = RecipeLevelTable::from_path("./csv/RecipeLevelTable.csv")?;
        let all_recipes = RecipeList::from_path("./csv/Recipe.csv")?;

        let mut map = BTreeMap::new();
        for (id, recipe_parsed) in all_recipes.0 {
            let level_info = all_recipe_levels
                .0
                .get(&recipe_parsed.level_id)
                .map(|level_info| RecipeLevelInfo {
                    level: level_info.level,
                    stars: level_info.stars,
                })
                .unwrap();

            map.insert(
                id,
                Recipe {
                    output: recipe_parsed.output,
                    inputs: recipe_parsed.inputs,
                    level: level_info.level,
                    stars: level_info.stars,
                },
            );
        }

        Ok(map)
    }

    async fn download_files() -> Result<()> {
        async fn download_url(url: &str) -> Result<String> {
            Ok(reqwest::get(url).await?.text().await?)
        }

        async fn download_file(file_name: &str) -> Result<()> {
            let local_path = format!("./csv/{file_name}");
            let url = &format!(
                "https://raw.githubusercontent.com/xivapi/ffxiv-datamining/master/csv/{file_name}"
            );

            if Path::exists(local_path.as_ref()) {
                return Ok(());
            }

            DirBuilder::new().recursive(true).create("./csv")?;

            println!("Downloading {local_path}");
            let content = download_url(url).await?;
            let mut writer = BufWriter::new(File::create(local_path)?);
            writer.write_all(content.as_ref())?;

            Ok(())
        }

        let files = [
            "ClassJobCategory.csv",
            "CraftLeve.csv",
            "Item.csv",
            "ItemUICategory.csv",
            "Leve.csv",
            "Recipe.csv",
            "RecipeLevelTable.csv",
        ];

        let results = join_all(files.into_iter().map(download_file)).await;
        results
            .into_iter()
            .filter_map(Result::err)
            .for_each(|err| panic!("{}", err));

        Ok(())
    }

    fn validate_items(&self) {
        for item in self.all_items.items.values() {
            if let Some(recipe) = &item.recipe {
                for input in &recipe.inputs {
                    if self.item_info_checked(input).is_none() {
                        println!("[Library] *WARN*: Item '{}' missing recipe references invalid item_id: {}", item.id, input.item_id);
                    }
                }
            }
        }
    }
}

impl Library {
    pub fn ui_category_unchecked(&self, ui_category: u32) -> &str {
        &self.all_ui_categories[&ui_category]
    }

    pub fn item_name<'a, I: ItemId>(&'a self, obj: &I) -> &'a str {
        let id = obj.item_id();
        &self.all_items[&id].name
    }

    pub fn item_info<'a, I: ItemId>(&'a self, obj: &I) -> &'a ItemInfo {
        let id = obj.item_id();
        &self.all_items[&id]
    }

    pub fn item_info_checked<'a, I: ItemId>(&'a self, obj: &I) -> Option<&'a ItemInfo> {
        self.all_items.items.get(&obj.item_id())
    }

    pub fn all_items(&self) -> Vec<&ItemInfo> {
        self.all_items.items.values().collect::<Vec<_>>()
    }
}
