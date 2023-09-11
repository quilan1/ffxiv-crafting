use anyhow::Result;
use futures::future::join_all;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use std::{fs::DirBuilder, io::Write};

use crate::recipe::RecipeLevelInfo;
use crate::Recipe;

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

static mut LIBRARY: Option<Library> = None;

pub(crate) fn library() -> &'static Library {
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

        library.all_crafting_leves = CraftLeveList::from_path("./csv/CraftLeve.csv")?;
        library.all_leves = LeveList::from_path(library, "./csv/Leve.csv")?;
        library.all_job_categories = JobCategoryList::from_path("./csv/ClassJobCategory.csv")?;

        let recipe_info = Self::parse_recipes()?;
        for (_, recipe) in recipe_info {
            if let Some(item) = library.all_items.items.get_mut(&recipe.output.item_id) {
                item.recipe = Some(recipe);
            }
        }

        Ok(())
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
                });

            map.insert(
                id,
                Recipe {
                    output: recipe_parsed.output,
                    inputs: recipe_parsed.inputs,
                    level_info,
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
}
