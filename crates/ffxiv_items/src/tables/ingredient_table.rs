use std::time::Instant;

use anyhow::Result;
use const_format::formatcp;
use futures::TryStreamExt;
use itertools::Itertools;
use sqlx::{QueryBuilder, Row};

use crate::{Ingredient, ItemDB, ItemId, Recipe};

use super::{strip_whitespace, RecipeTable, BIND_MAX};

////////////////////////////////////////////////////////////

impl_table!(IngredientTable);

impl IngredientTable<'_> {
    pub async fn initialize(&self, recipes: &[Recipe]) -> Result<()> {
        println!("Initializing Ingredients Database Table");
        let ingredients = recipes
            .iter()
            .map(|recipe| (recipe.output.item_id, recipe))
            .flat_map(|(item_id, recipe)| {
                recipe
                    .inputs
                    .iter()
                    .map(move |ingredient| (item_id, ingredient))
            });

        for ingredients in &ingredients.chunks(BIND_MAX / 3) {
            QueryBuilder::new(strip_whitespace(SQL_INSERT))
                .push_values(ingredients, |mut b, (item_id, recipe)| {
                    b.push_bind(item_id)
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
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let start = Instant::now();
        let num_ids = ids.len();
        let ids = ids.iter().map(|id| id.item_id().to_string()).join(",");
        let query_string = strip_whitespace(format!("{} ({ids})", SQL_SELECT));

        let mut ingredients = Vec::new();
        let mut sql_query = sqlx::query(&query_string).persistent(true).fetch(self.db);
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

        log::debug!(target: "ffxiv_items", "Query for {num_ids} ingredients ({} returned): {:.3}s", ingredients.len(), start.elapsed().as_secs_f32());
        Ok(ingredients)
    }
}

////////////////////////////////////////////////////////////

pub const SQL_TABLE_NAME: &str = "ingredients";

const SQL_CREATE: &str = formatcp!(
    "CREATE TABLE IF NOT EXISTS {SQL_TABLE_NAME} (
        id          MEDIUMINT   UNSIGNED    AUTO_INCREMENT  PRIMARY KEY,
        item_id     MEDIUMINT   UNSIGNED    NOT NULL,
        input_id    MEDIUMINT   UNSIGNED    NOT NULL,
        count       SMALLINT    UNSIGNED    NOT NULL,
        INDEX       ( item_id ),
        INDEX       ( input_id ),
        INDEX       input_item  ( input_id, item_id )
    )"
);

const SQL_INSERT: &str = formatcp!("INSERT INTO {SQL_TABLE_NAME} (item_id, input_id, count) ");

const SQL_SELECT: &str = formatcp!(
    "SELECT r.id, g.input_id, g.count
    FROM {SQL_TABLE_NAME} AS g
    INNER JOIN {} as r ON r.id = g.item_id
    WHERE r.id IN",
    RecipeTable::SQL_TABLE_NAME
);
