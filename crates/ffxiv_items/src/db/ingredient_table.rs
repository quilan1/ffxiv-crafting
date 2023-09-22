use anyhow::Result;
use const_format::formatcp;
use futures::TryStreamExt;
use itertools::Itertools;
use sqlx::{QueryBuilder, Row};

use crate::{Ingredient, ItemId, Recipe};

use super::{ItemDB, RecipeTable, BIND_MAX};

////////////////////////////////////////////////////////////

impl_table!(IngredientTable);

impl IngredientTable<'_> {
    pub async fn initialize(&self, recipes: &[Recipe]) -> Result<()> {
        if !self.is_empty().await? {
            return Ok(());
        }

        println!("Initializing Ingredients Database Table");

        let ingredients = recipes
            .iter()
            .enumerate()
            .map(|(index, recipe)| (index, recipe))
            .flat_map(|(index, recipe)| {
                recipe
                    .inputs
                    .iter()
                    .map(move |ingredient| (index, ingredient))
            });

        for ingredients in &ingredients.chunks(BIND_MAX / 3) {
            QueryBuilder::new(SQL_INSERT)
                .push_values(ingredients, |mut b, (item_id, recipe)| {
                    b.push_bind(item_id as u64 + 1)
                        .push_bind(recipe.item_id)
                        .push_bind(recipe.count);
                })
                .build()
                .execute(self.db)
                .await?;
        }

        Ok(())
    }

    pub async fn by_item_ids<I: ItemId>(&self, ids: &[I]) -> Result<Vec<(u32, Ingredient)>> {
        let ids = ids.iter().map(|id| id.item_id().to_string()).join(",");
        let query_string = format!("{} ({ids})", SQL_SELECT);

        let mut ingredients = Vec::new();
        let mut sql_query = sqlx::query(&query_string).fetch(self.db);
        while let Some(row) = sql_query.try_next().await? {
            let item_id: u64 = row.get(0);
            let input_id: u64 = row.get(1);
            let count: u64 = row.get(2);
            ingredients.push((
                item_id as u32,
                Ingredient {
                    count: count as u32,
                    item_id: input_id as u32,
                },
            ));
        }

        Ok(ingredients)
    }
}

////////////////////////////////////////////////////////////

pub const SQL_TABLE_NAME: &str = "ingredients";

const SQL_CREATE: &str = formatcp!(
    "CREATE TABLE IF NOT EXISTS {SQL_TABLE_NAME} (
        id          MEDIUMINT   UNSIGNED    AUTO_INCREMENT,
        recipe_id   SMALLINT    UNSIGNED    NOT NULL,
        item_id     MEDIUMINT   UNSIGNED    NOT NULL,
        count       SMALLINT    UNSIGNED    NOT NULL,
        PRIMARY KEY     ( id ),
        INDEX       id0 ( recipe_id ),
        INDEX       id1 ( item_id )
    )"
);

const SQL_EMPTY: &str = formatcp!("SELECT COUNT(id) FROM {SQL_TABLE_NAME}");

const SQL_INSERT: &str = formatcp!("INSERT INTO {SQL_TABLE_NAME} (recipe_id, item_id, count) ");

const SQL_SELECT: &str = formatcp!(
    "SELECT r.item_id, g.item_id, g.count
    FROM {SQL_TABLE_NAME} AS g
    INNER JOIN {} as r
    ON r.id = g.recipe_id
    WHERE r.item_id IN",
    RecipeTable::SQL_TABLE_NAME
);
