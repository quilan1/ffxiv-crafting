use std::{collections::BTreeMap, time::Instant};

use anyhow::Result;
use const_format::formatcp;
use futures::TryStreamExt;
use itertools::Itertools;
use sqlx::{QueryBuilder, Row};

use crate::{Ingredient, ItemDB, ItemId, Recipe};

use super::{IngredientTable, BIND_MAX};

////////////////////////////////////////////////////////////

impl_table!(RecipeTable);

impl RecipeTable<'_> {
    pub async fn initialize(&self, recipes: &[Recipe]) -> Result<()> {
        if !self.is_empty().await? {
            return Ok(());
        }

        println!("Initializing Recipes Database Table");

        let recipes = recipes.iter();
        for recipes in &recipes.chunks(BIND_MAX / 4) {
            QueryBuilder::new(SQL_INSERT)
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

    pub async fn by_item_ids<I: ItemId>(&self, ids: &[I]) -> Result<Vec<Recipe>> {
        let start = Instant::now();
        let _ids = ids.iter().map(|id| id.item_id().to_string()).join(",");
        let query_string = format!("{} ({_ids})", SQL_SELECT);

        let mut recipes = BTreeMap::new();
        let mut sql_query = sqlx::query(&query_string).fetch(self.db);
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

        let ingredients = IngredientTable::new(self.db).by_item_ids(ids).await?;
        for (item_id, ingredient) in ingredients {
            recipes.entry(item_id).and_modify(|recipe| {
                recipe.inputs.push(ingredient);
            });
        }

        log::debug!(target: "ffxiv_items", "Query for {} recipes: {:.3}s", ids.len(), start.elapsed().as_secs_f32());
        Ok(recipes.into_values().collect())
    }
}

////////////////////////////////////////////////////////////

const SQL_TABLE_NAME: &str = "recipes";

const SQL_CREATE: &str = formatcp!(
    "CREATE TABLE IF NOT EXISTS {SQL_TABLE_NAME} (
        id      MEDIUMINT   UNSIGNED    AUTO_INCREMENT,
        item_id MEDIUMINT   UNSIGNED    NOT NULL    UNIQUE,
        count   SMALLINT    UNSIGNED    NOT NULL,
        level   SMALLINT    UNSIGNED    NOT NULL,
        stars   SMALLINT    UNSIGNED    NOT NULL,
        PRIMARY     KEY ( id ),
        INDEX       id0 ( item_id )
    )"
);

const SQL_EMPTY: &str = formatcp!("SELECT COUNT(id) FROM {SQL_TABLE_NAME}");

const SQL_INSERT: &str = formatcp!("INSERT INTO {SQL_TABLE_NAME} (item_id, count, level, stars) ");

const SQL_SELECT: &str = formatcp!(
    "SELECT item_id, count, level, stars
    FROM {SQL_TABLE_NAME}
    WHERE item_id IN"
);
