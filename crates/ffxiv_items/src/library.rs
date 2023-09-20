use anyhow::Result;
use futures::join;
use reqwest::IntoUrl;
use std::collections::BTreeMap;
use std::io::Cursor;

use crate::parsers::{
    CraftLeveList, ItemList, JobCategoryList, LeveList, RecipeLevelTable, RecipeList,
    UiCategoryList,
};
use crate::{ItemId, ItemInfo, Recipe, RecipeLevelInfo};

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

        let CsvContent {
            item,
            item_ui_category,
            class_job_category,
            craft_leve,
            leve,
            recipe,
            recipe_level_table,
        } = CsvContent::download().await?;

        library.all_items = ItemList::from_reader(item)?;
        library.all_ui_categories = UiCategoryList::from_reader(item_ui_category)?;

        library.all_job_categories = JobCategoryList::from_reader(class_job_category)?;
        library.all_crafting_leves = CraftLeveList::from_reader(craft_leve)?;
        library.all_leves = LeveList::from_reader(leve, &library)?;

        let recipe_info = Self::parse_recipes(recipe, recipe_level_table)?;
        for (_, recipe) in recipe_info {
            if let Some(item) = library.all_items.items.get_mut(&recipe.output.item_id) {
                item.recipe = Some(recipe);
            }
        }

        library.validate_items();

        Ok(library)
    }

    fn parse_recipes(
        recipes: Cursor<String>,
        recipe_levels: Cursor<String>,
    ) -> Result<BTreeMap<u32, Recipe>> {
        let all_recipes = RecipeList::from_reader(recipes)?;
        let all_recipe_levels = RecipeLevelTable::from_reader(recipe_levels)?;

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

struct CsvContent {
    item: Cursor<String>,
    item_ui_category: Cursor<String>,

    class_job_category: Cursor<String>,
    craft_leve: Cursor<String>,
    leve: Cursor<String>,

    recipe: Cursor<String>,
    recipe_level_table: Cursor<String>,
}

impl CsvContent {
    async fn download() -> Result<Self> {
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
            recipe_level_table,
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
