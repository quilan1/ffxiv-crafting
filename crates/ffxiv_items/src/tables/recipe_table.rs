use std::{collections::BTreeMap, io::Cursor, time::Instant};

use anyhow::Result;
use chrono::{DateTime, FixedOffset};
use const_format::formatcp;
use futures::{try_join, TryStreamExt};
use itertools::Itertools;
use mock_traits::FileDownloader;
use sqlx::{QueryBuilder, Row};

use crate::{csv_parse, last_updated_from_github, Ingredient, ItemDB, ItemId, Recipe};

use super::{
    download_csv, impl_table, impl_table_builder, strip_whitespace, IngredientTable, BIND_MAX,
};

////////////////////////////////////////////////////////////

impl_table!(RecipeTable);
impl_table_builder!(RecipeTableBuilder, FileDownloader);

impl RecipeTable<'_> {
    pub async fn by_item_ids<I: ItemId>(&self, ids: &[I]) -> Result<Vec<Recipe>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let start = Instant::now();
        let _ids = ids.iter().map(|id| id.item_id().to_string()).join(",");
        let query_string = strip_whitespace(format!("{} ({_ids})", SQL_SELECT));

        let mut recipes = BTreeMap::new();
        let mut sql_query = sqlx::query(&query_string).persistent(true).fetch(self.db);
        while let Some(row) = sql_query.try_next().await? {
            let item_id: u32 = row.get(0);
            let count: u32 = row.get(1);
            let level: u32 = row.get(2);
            let stars: u32 = row.get(3);
            recipes.insert(
                item_id,
                Recipe {
                    output: Ingredient { count, item_id },
                    inputs: Vec::new(),
                    level,
                    stars,
                },
            );
        }
        log::debug!(target: "ffxiv_items", "Query for {} recipes: {:.3}s", ids.len(), start.elapsed().as_secs_f32());

        let ingredients = IngredientTable::new(self.db).by_item_ids(ids).await?;
        for (item_id, ingredient) in ingredients {
            recipes.entry(item_id).and_modify(|recipe| {
                recipe.inputs.push(ingredient);
            });
        }

        Ok(recipes.into_values().collect())
    }
}

////////////////////////////////////////////////////////////

pub struct CsvRecipe {
    pub output: Ingredient,
    pub inputs: Vec<Ingredient>,
    pub level_id: u32,
}

pub struct CsvRecipeLevel {
    pub id: u32,
    pub level: u32,
    pub stars: u32,
}

////////////////////////////////////////////////////////////

const CSV_FILE_RECIPE: &str = "Recipe.csv";
const CSV_FILE_RECIPE_LEVEL: &str = "RecipeLevelTable.csv";

impl<F: FileDownloader> RecipeTableBuilder<'_, F> {
    pub async fn initialize(&self, recipes: &[Recipe]) -> Result<()> {
        println!("Initializing Recipes Database Table");
        for recipes in &recipes.iter().chunks(BIND_MAX / 4) {
            QueryBuilder::new(strip_whitespace(SQL_INSERT))
                .push_values(recipes, |mut b, recipe| {
                    b.push_bind(recipe.output.item_id)
                        .push_bind(recipe.output.count)
                        .push_bind(recipe.level)
                        .push_bind(recipe.stars);
                })
                .build()
                .execute(self.db)
                .await?;
        }

        Ok(())
    }

    pub async fn last_updated_github(&self) -> Result<DateTime<FixedOffset>> {
        let recipe_updated = last_updated_from_github::<F>(CSV_FILE_RECIPE).await?;
        let recipe_level_updated = last_updated_from_github::<F>(CSV_FILE_RECIPE_LEVEL).await?;
        Ok(recipe_updated.max(recipe_level_updated))
    }

    pub async fn download_recipe_info() -> Result<Vec<Recipe>> {
        println!("Downloading Recipes from Github");

        let (csv_recipes, csv_recipe_levels) = try_join!(
            Self::download_recipe_csv(),
            Self::download_recipe_level_csv()
        )?;

        let recipes = csv_recipes
            .into_iter()
            .map(|csv_recipe| {
                let recipe_level = &csv_recipe_levels[&csv_recipe.level_id];
                Recipe {
                    output: csv_recipe.output,
                    inputs: csv_recipe.inputs,
                    level: recipe_level.level,
                    stars: recipe_level.stars,
                }
            })
            .collect();

        Ok(recipes)
    }

    async fn download_recipe_csv() -> Result<Vec<CsvRecipe>> {
        let reader = Cursor::new(download_csv::<F>(CSV_FILE_RECIPE).await?);
        let mut recipes = BTreeMap::new();
        csv_parse!(reader => {
            level_id = U[2 + 1];
            arr = U[4..24];

            let mut ingredients = Vec::new();
            for (item_id, count) in arr.into_iter().tuples() {
                if count > 0 {
                    ingredients.push(Ingredient { count, item_id });
                }
            }

            if ingredients.is_empty() || ingredients[0].item_id == 0 {
                continue;
            }

            let output = ingredients.remove(0);
            let inputs = ingredients;
            recipes.insert(output.item_id,
                CsvRecipe {
                    output,
                    inputs,
                    level_id,
                },
            );
        });

        Ok(recipes.into_values().collect_vec())
    }

    async fn download_recipe_level_csv() -> Result<BTreeMap<u32, CsvRecipeLevel>> {
        let reader = Cursor::new(download_csv::<F>(CSV_FILE_RECIPE_LEVEL).await?);
        let mut recipe_levels = BTreeMap::new();
        csv_parse!(reader => {
            id = U[0];
            level = U[1];
            stars = U[1 + 1];
            recipe_levels.insert(id, CsvRecipeLevel { id, level, stars });
        });

        Ok(recipe_levels)
    }
}

////////////////////////////////////////////////////////////

const SQL_TABLE_NAME: &str = "recipes";

const SQL_CREATE: &str = formatcp!(
    "CREATE TABLE IF NOT EXISTS {SQL_TABLE_NAME} (
        id      MEDIUMINT   UNSIGNED    NOT NULL    PRIMARY KEY,
        count   SMALLINT    UNSIGNED    NOT NULL,
        level   SMALLINT    UNSIGNED    NOT NULL,
        stars   SMALLINT    UNSIGNED    NOT NULL,
        INDEX   ( level )
    )"
);

const SQL_INSERT: &str = formatcp!("INSERT INTO {SQL_TABLE_NAME} (id, count, level, stars) ");

const SQL_SELECT: &str =
    formatcp!("SELECT id, count, level, stars FROM {SQL_TABLE_NAME} WHERE id IN");
