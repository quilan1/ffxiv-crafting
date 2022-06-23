use std::collections::HashSet;

use anyhow::Result;

use crate::universalis::Universalis;
use crate::{RunMode, Settings};

use super::parsers::*;
use super::CraftList;

#[derive(Default)]
pub struct Library {
    pub all_items: ItemList,
    pub all_ui_categories: UiCategoryList,
    pub all_recipe_levels: RecipeLevelTable,
    pub all_recipes: RecipeList,
    pub all_gathering_levels: GatheringLevelList,
    pub all_gathering: GatheringList,
    pub all_crafts: CraftList,
    pub all_custom_crafts: CraftList,
    pub all_crafting_leves: CraftLeveList,
    pub all_leves: LeveList,
    pub all_job_categories: JobCategoryList,
}

impl Library {
    pub fn new() -> Result<Self> {
        let mut library = Self::default();

        library.all_items = ItemList::from_path("./Item.csv")?;
        library.all_ui_categories = UiCategoryList::from_path("./ItemUICategory.csv")?;
        library.all_recipe_levels = RecipeLevelTable::from_path("./RecipeLevelTable.csv")?;
        library.all_recipes = RecipeList::from_path("./Recipe.csv")?;
        library.all_gathering_levels =
            GatheringLevelList::from_path("./GatheringItemLevelConvertTable.csv")?;
        library.all_gathering = GatheringList::from_path("./GatheringItem.csv", &library)?;
        library.all_job_categories = JobCategoryList::from_path("./ClassJobCategory.csv")?;
        library.all_crafting_leves = CraftLeveList::from_path("./CraftLeve.csv")?;
        library.all_leves = LeveList::from_path("./Leve.csv", &library)?;
        library.all_crafts = CraftList::from_path("./in_crafting_list.txt", &library, false)?;
        library.all_custom_crafts = CraftList::from_path("./in_custom_list.txt", &library, true)?;

        Ok(library)
    }

    pub fn all_craftable_items(&self) -> Vec<&ItemInfo> {
        self.all_items.all_craftable_items(self)
    }

    pub fn all_gatherable_items(&self) -> Vec<&ItemInfo> {
        self.all_items.all_gatherable_items(self)
    }

    pub fn all_market_board_ids(&self, settings: &Settings) -> Vec<u32> {
        let mut ids = HashSet::new();
        if [RunMode::OnlyCrafting, RunMode::All].contains(&settings.run_mode) {
            ids.extend(self.all_crafts.all_craft_item_ids(self));
        }
        if [RunMode::OnlyCustom, RunMode::All].contains(&settings.run_mode) {
            ids.extend(self.all_custom_crafts.all_craft_item_ids(self));
        }
        if [RunMode::OnlyGathering, RunMode::All].contains(&settings.run_mode) {
            ids.extend(self.all_gatherable_items().iter().map(|item| item.id));
        }
        ids.into_iter().collect::<Vec<_>>()
    }

    pub fn write_files(&self, universalis: &Universalis, settings: &Settings) -> Result<()> {
        if [RunMode::OnlyCrafting, RunMode::All].contains(&settings.run_mode) {
            self.all_crafts
                .write_to_file("./out_crafts.txt", self, universalis, settings)?;
        }
        if [RunMode::OnlyCustom, RunMode::All].contains(&settings.run_mode) {
            self.all_custom_crafts.write_custom_to_file(
                "./out_custom.txt",
                self,
                universalis,
                settings,
            )?;
        }
        if [RunMode::OnlyGathering, RunMode::All].contains(&settings.run_mode) {
            self.all_gathering
                .write_to_file("./out_gathering.txt", self, universalis, settings)?;
        }
        Ok(())
    }
}
