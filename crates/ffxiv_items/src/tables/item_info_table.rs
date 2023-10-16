use std::{collections::BTreeMap, io::Cursor, time::Instant};

use anyhow::Result;
use chrono::{DateTime, FixedOffset};
use const_format::formatcp;
use futures::TryStreamExt;
use itertools::Itertools;
use sqlx::{QueryBuilder, Row};

use crate::{csv_parse, last_updated_from_github, ItemDB, ItemId, ItemInfo};

use super::{download_file, strip_whitespace, RecipeTable, BIND_MAX};

////////////////////////////////////////////////////////////

impl_table!(ItemInfoTable);

impl ItemInfoTable<'_> {
    pub async fn by_item_ids<I: ItemId>(&self, ids: &[I]) -> Result<Vec<ItemInfo>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let start = Instant::now();
        let _ids = ids.iter().map(|id| id.item_id().to_string()).join(",");
        let query_string = strip_whitespace(format!("{} ({_ids})", SQL_SELECT));

        let mut items = BTreeMap::new();
        let mut sql_query = sqlx::query(&query_string).persistent(true).fetch(self.db);
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
        log::debug!(target: "ffxiv_items", "Query for {} items: {:.3}s", ids.len(), start.elapsed().as_secs_f32());

        let recipes = RecipeTable::new(self.db).by_item_ids(ids).await?;
        for recipe in recipes {
            items.entry(recipe.output.item_id).and_modify(|item| {
                item.recipe = Some(recipe);
            });
        }

        Ok(items.into_values().collect())
    }
}

////////////////////////////////////////////////////////////

struct CsvItem {
    pub id: u32,
    pub name: String,
    pub ui_category: u32,
    pub ilevel: u32,
    pub equip_level: u32,
}

////////////////////////////////////////////////////////////

const CSV_FILE: &str = "Item.csv";

impl ItemInfoTable<'_> {
    pub async fn initialize(&self) -> Result<()> {
        let items = Self::download().await?;

        println!("Initializing Items Database Table");
        let items = items.iter().filter(|item| !item.name.is_empty());
        for items in &items.chunks(BIND_MAX / 5) {
            QueryBuilder::new(strip_whitespace(SQL_INSERT))
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

    pub async fn last_updated_github() -> Result<DateTime<FixedOffset>> {
        last_updated_from_github(CSV_FILE).await
    }

    async fn download() -> Result<Vec<CsvItem>> {
        println!("Downloading Items from Github");

        let reader = Cursor::new(download_file(CSV_FILE).await?);
        let mut items = Vec::new();
        csv_parse!(reader => {
            id = U[0];
            name = S[9 + 1];
            ilevel = U[11 + 1];
            ui_category = U[15 + 1];
            equip_level = U[40 + 1];

            let item = CsvItem {
                id,
                name: name.replace('\u{00A0}', " ").clone(),
                ui_category,
                ilevel,
                equip_level,
            };

            items.push(item);
        });

        Ok(items)
    }
}

////////////////////////////////////////////////////////////

const SQL_TABLE_NAME: &str = "items";

const SQL_CREATE: &str = formatcp!(
    "CREATE TABLE IF NOT EXISTS {SQL_TABLE_NAME} (
        id          MEDIUMINT       UNSIGNED    PRIMARY KEY,
        name        VARCHAR(100)                NOT NULL,
        ui_category SMALLINT        UNSIGNED    NOT NULL,
        item_level  SMALLINT        UNSIGNED    NOT NULL,
        equip_level SMALLINT        UNSIGNED    NOT NULL,
        INDEX       ( name ),
        INDEX       ( item_level ),
        INDEX       ( equip_level ),
        INDEX       ( ui_category )
    )"
);

const SQL_INSERT: &str =
    formatcp!("INSERT INTO {SQL_TABLE_NAME} (id, name, ui_category, item_level, equip_level) ");

const SQL_SELECT: &str = formatcp!("SELECT id, name FROM {SQL_TABLE_NAME} WHERE id IN");
