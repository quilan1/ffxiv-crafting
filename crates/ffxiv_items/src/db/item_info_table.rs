use std::{collections::BTreeMap, time::Instant};

use anyhow::Result;
use const_format::formatcp;
use futures::TryStreamExt;
use itertools::Itertools;
use sqlx::{QueryBuilder, Row};

use crate::{parsers, ItemId, ItemInfo};

use super::{ItemDB, RecipeTable, BIND_MAX};

////////////////////////////////////////////////////////////

impl_table!(ItemInfoTable);

impl ItemInfoTable<'_> {
    pub async fn initialize(&self, items: &parsers::ItemList) -> Result<()> {
        if !self.is_empty().await? {
            return Ok(());
        }

        println!("Initializing Items Database Table");

        let items = items.0.iter().filter(|item| !item.name.is_empty());
        for items in &items.chunks(BIND_MAX / 5) {
            QueryBuilder::new(SQL_INSERT)
                .push_values(items, |mut b, item| {
                    b.push_bind(item.id)
                        .push_bind(&item.name)
                        .push_bind(item.ui_category)
                        .push_bind(item.ilevel)
                        .push_bind(item.equip_level);
                })
                .build()
                .execute(self.db)
                .await?;
        }

        Ok(())
    }

    pub async fn by_item_ids<I: ItemId>(&self, ids: &[I]) -> Result<Vec<ItemInfo>> {
        let start = Instant::now();
        let _ids = ids.iter().map(|id| id.item_id().to_string()).join(",");
        let query_string = format!("{} ({_ids})", SQL_SELECT);

        let mut items = BTreeMap::new();
        let mut sql_query = sqlx::query(&query_string).fetch(self.db);
        while let Some(row) = sql_query.try_next().await? {
            let item_id: u32 = row.get(0);
            let name: String = row.get(1);
            items.insert(
                item_id,
                ItemInfo {
                    id: item_id,
                    name,
                    recipe: None,
                },
            );
        }

        let recipes = RecipeTable::new(self.db).by_item_ids(ids).await?;
        for recipe in recipes {
            items.entry(recipe.output.item_id).and_modify(|item| {
                item.recipe = Some(recipe);
            });
        }

        log::debug!(target: "ffxiv_items", "Query for {} items: {:.3}s", ids.len(), start.elapsed().as_secs_f32());
        Ok(items.into_values().collect())
    }
}

////////////////////////////////////////////////////////////

const SQL_TABLE_NAME: &str = "items";

const SQL_CREATE: &str = formatcp!(
    "CREATE TABLE IF NOT EXISTS {SQL_TABLE_NAME} (
        id          MEDIUMINT       UNSIGNED,
        name        VARCHAR(100)    NOT NULL,
        ui_category SMALLINT        UNSIGNED    NOT NULL,
        item_level  SMALLINT        UNSIGNED    NOT NULL,
        equip_level SMALLINT        UNSIGNED    NOT NULL,
        PRIMARY KEY ( id )
    )"
);

const SQL_EMPTY: &str = formatcp!("SELECT COUNT(id) FROM {SQL_TABLE_NAME}");

const SQL_INSERT: &str =
    formatcp!("INSERT INTO {SQL_TABLE_NAME} (id, name, ui_category, item_level, equip_level) ");

const SQL_SELECT: &str = formatcp!(
    "SELECT id, name
    FROM {SQL_TABLE_NAME}
    WHERE id IN"
);
