use anyhow::Result;
use std::collections::BTreeMap;
use std::io::Cursor;

use crate::parsers::{
    CraftLeveList, ItemList, JobCategoryList, LeveList, RecipeLevelTable, RecipeList,
    UiCategoryList,
};
use crate::{CsvContent, ItemId, ItemInfo, Recipe, RecipeLevelInfo};

////////////////////////////////////////////////////////////

#[derive(Default)]
pub struct Library {
    pub(crate) all_items: ItemList,
    pub(crate) all_ui_categories: UiCategoryList,
    pub(crate) all_crafting_leves: CraftLeveList,
    pub(crate) all_leves: LeveList,
    pub(crate) all_job_categories: JobCategoryList,
}

////////////////////////////////////////////////////////////

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
            recipe_level: recipe_level_table,
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

////////////////////////////////////////////////////////////

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
