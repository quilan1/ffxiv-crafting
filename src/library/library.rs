use anyhow::Result;
use std::{collections::HashSet, fs::DirBuilder, io::Write};

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

        library.all_items = ItemList::from_path("./csv/Item.csv")?;
        library.all_ui_categories = UiCategoryList::from_path("./csv/ItemUICategory.csv")?;
        library.all_recipe_levels = RecipeLevelTable::from_path("./csv/RecipeLevelTable.csv")?;
        library.all_recipes = RecipeList::from_path("./csv/Recipe.csv")?;
        library.all_gathering_levels =
            GatheringLevelList::from_path("./csv/GatheringItemLevelConvertTable.csv")?;
        library.all_gathering = GatheringList::from_path("./csv/GatheringItem.csv", &library)?;
        library.all_job_categories = JobCategoryList::from_path("./csv/ClassJobCategory.csv")?;
        library.all_crafting_leves = CraftLeveList::from_path("./csv/CraftLeve.csv")?;
        library.all_leves = LeveList::from_path("./csv/Leve.csv", &library)?;

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
        DirBuilder::new().recursive(true).create("./out")?;
        if [RunMode::OnlyCrafting, RunMode::All].contains(&settings.run_mode) {
            self.all_crafts
                .write_to_file("./out/crafts.txt", self, universalis, settings)?;
        }
        if [RunMode::OnlyCustom, RunMode::All].contains(&settings.run_mode) {
            self.all_custom_crafts.write_custom_to_file(
                "./out/custom.txt",
                self,
                universalis,
                settings,
            )?;
        }
        if [RunMode::OnlyGathering, RunMode::All].contains(&settings.run_mode) {
            self.all_gathering
                .write_to_file("./out/gathering.txt", self, universalis, settings)?;
        }
        self.write_outbid(universalis, settings, "./out/bids.txt")?;
        Ok(())
    }

    fn write_outbid<P: AsRef<std::path::Path>>(
        &self,
        universalis: &Universalis,
        settings: &Settings,
        path: P,
    ) -> Result<()> {
        let mut writer = std::io::BufWriter::new(std::fs::File::create(path.as_ref())?);

        write!(
            &mut writer,
            "{:<40}| {:<30}| {:<10}{:<10}\n",
            "Name", "Seller", "Count", "Price"
        )?;

        for (item_id, mb_info) in &universalis.homeworld {
            if !mb_info
                .listings
                .iter()
                .any(|listing| settings.characters.contains(&listing.name))
            {
                continue;
            }

            for listing in &mb_info.listings {
                let time = std::time::SystemTime::UNIX_EPOCH
                    + std::time::Duration::from_secs(listing.posting);
                write!(
                    &mut writer,
                    "{:<40}| {:<30}| {:<10}{:<10}{:<10.1}\n",
                    self.all_items[item_id].name,
                    listing.name,
                    listing.count,
                    listing.price,
                    time.elapsed()
                        .unwrap_or(std::time::Duration::from_secs(0))
                        .as_secs_f32()
                        / (3600.0 * 24.0),
                )?;

                if settings.characters.contains(&listing.name) {
                    break;
                }
            }

            write!(&mut writer, "\n")?;
        }

        Ok(())
    }
}
