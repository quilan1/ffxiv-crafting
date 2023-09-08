use anyhow::Result;
use futures::future::join_all;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use std::{collections::HashSet, fs::DirBuilder, io::Write};

// use crate::cli::{settings, RunMode};

use super::parsers::{
    CraftLeveList, GatheringLevelList, GatheringList, ItemInfo, ItemList, JobCategoryList,
    LeveList, RecipeLevelTable, RecipeList, UiCategoryList,
};
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

static mut LIBRARY: Option<Library> = None;

pub fn library() -> &'static Library {
    unsafe { LIBRARY.as_ref().expect("LIBRARY has not been set!") }
}

fn library_mut() -> &'static mut Library {
    unsafe { LIBRARY.as_mut().expect("LIBRARY has not been set!") }
}

impl Library {
    pub async fn create() -> Result<()> {
        unsafe {
            LIBRARY = Some(Library::default());
        }

        let library = library_mut();

        Self::download_files().await?;

        library.all_items = ItemList::from_path("./csv/Item.csv")?;
        library.all_ui_categories = UiCategoryList::from_path("./csv/ItemUICategory.csv")?;
        library.all_recipe_levels = RecipeLevelTable::from_path("./csv/RecipeLevelTable.csv")?;
        library.all_recipes = RecipeList::from_path("./csv/Recipe.csv")?;
        library.all_gathering_levels =
            GatheringLevelList::from_path("./csv/GatheringItemLevelConvertTable.csv")?;
        library.all_gathering = GatheringList::from_path("./csv/GatheringItem.csv")?;
        library.all_job_categories = JobCategoryList::from_path("./csv/ClassJobCategory.csv")?;
        library.all_crafting_leves = CraftLeveList::from_path("./csv/CraftLeve.csv")?;
        library.all_leves = LeveList::from_path("./csv/Leve.csv")?;

        library.all_crafts = CraftList::from_path("./in_crafting_list.txt", false)?;
        library.all_custom_crafts = CraftList::from_path("./in_custom_list.txt", true)?;

        Ok(())
    }

    pub async fn download_files() -> Result<()> {
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
            "GatheringItem.csv",
            "GatheringItemLevelConvertTable.csv",
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

    pub fn all_craftable_items(&self) -> Vec<&ItemInfo> {
        self.all_items.all_craftable_items()
    }

    #[allow(dead_code)]
    pub fn all_gatherable_items(&self) -> Vec<&ItemInfo> {
        self.all_items.all_gatherable_items()
    }

    #[allow(dead_code)]
    pub fn all_market_board_ids(&self) -> Vec<u32> {
        let mut ids = HashSet::new();
        ids.extend(self.all_crafts.all_craft_item_ids());
        ids.extend(self.all_custom_crafts.all_craft_item_ids());
        ids.extend(self.all_gatherable_items().iter().map(|item| item.id));
        ids.into_iter().collect::<Vec<_>>()
    }
}
