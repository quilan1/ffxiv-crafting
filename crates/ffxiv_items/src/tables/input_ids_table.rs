use std::{collections::BTreeMap, time::Instant};

use anyhow::Result;
use const_format::formatcp;
use futures::TryStreamExt;
use itertools::Itertools;
use sqlx::{QueryBuilder, Row};

use crate::{ItemDB, ItemId, Recipe};

use super::BIND_MAX;

////////////////////////////////////////////////////////////

impl_table!(InputIdsTable);

impl InputIdsTable<'_> {
    pub async fn initialize(&self, recipes: &[Recipe]) -> Result<()> {
        println!("Initializing Input IDs Database Table");

        let recipes = Recipe::to_map_ref(recipes);
        let id_map = recipes
            .iter()
            .flat_map(|(_, recipe)| {
                from_item_id(recipe.output.item_id, &recipes)
                    .into_iter()
                    .map(|input_id| (recipe.output.item_id, input_id))
            })
            .collect::<Vec<_>>();

        for id_map in id_map.chunks(BIND_MAX / 2) {
            QueryBuilder::new(SQL_INSERT)
                .push_values(id_map, |mut b, &(item_id, input_id)| {
                    b.push_bind(item_id).push_bind(input_id);
                })
                .build()
                .execute(self.db)
                .await?;
        }

        Ok(())
    }

    pub async fn by_item_ids<I: ItemId>(&self, ids: &[I]) -> Result<Vec<u32>> {
        let start = Instant::now();
        let num_ids = ids.len();
        let ids = ids.iter().map(|id| id.item_id().to_string()).join(",");
        let query_string = format!("{} ({ids})", SQL_SELECT);

        let mut input_ids = Vec::new();
        let mut sql_query = sqlx::query(&query_string).fetch(self.db);
        while let Some(row) = sql_query.try_next().await? {
            let input_id: u32 = row.get(0);
            input_ids.push(input_id);
        }
        log::debug!(target: "ffxiv_items", "Query for {num_ids} input ids ({} returned): {:.3}s", input_ids.len(), start.elapsed().as_secs_f32());
        Ok(input_ids)
    }
}

fn from_item_id<I: ItemId>(id: I, recipes: &BTreeMap<u32, &Recipe>) -> Vec<u32> {
    fn push_ids(recipes: &BTreeMap<u32, &Recipe>, ids: &mut Vec<u32>, item_id: u32) {
        ids.push(item_id);

        if let Some(recipe) = recipes.get(&item_id) {
            for input in &recipe.inputs {
                push_ids(recipes, ids, input.item_id);
            }
        }
    }

    let mut ids = Vec::new();
    push_ids(recipes, &mut ids, id.item_id());
    let mut ids = ids.into_iter().unique().collect_vec();
    ids.sort();
    ids
}

////////////////////////////////////////////////////////////

const SQL_TABLE_NAME: &str = "input_ids";

const SQL_CREATE: &str = formatcp!(
    "CREATE TABLE IF NOT EXISTS {SQL_TABLE_NAME} (
        id          MEDIUMINT   UNSIGNED    AUTO_INCREMENT,
        item_id     MEDIUMINT   UNSIGNED    NOT NULL,
        input_id    SMALLINT    UNSIGNED    NOT NULL,
        PRIMARY KEY     ( id ),
        INDEX       id0 ( item_id ),
        INDEX       id1 ( input_id )
    )"
);

const SQL_INSERT: &str = formatcp!("INSERT INTO {SQL_TABLE_NAME} (item_id, input_id) ");

const SQL_SELECT: &str = formatcp!(
    "SELECT DISTINCT input_id
    FROM {SQL_TABLE_NAME}
    WHERE item_id IN"
);
