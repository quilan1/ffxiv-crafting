use std::io::Cursor;

use anyhow::Result;
use const_format::formatcp;
use itertools::Itertools;
use sqlx::QueryBuilder;

use crate::ItemDB;

use super::{download_file, BIND_MAX};

////////////////////////////////////////////////////////////

struct CsvUiCategory {
    id: u32,
    name: String,
}

////////////////////////////////////////////////////////////

const CSV_FILE: &str = "ItemUICategory.csv";

impl_table!(UiCategoryTable);

impl UiCategoryTable<'_> {
    pub async fn initialize(&self) -> Result<()> {
        if !self.is_empty().await? {
            return Ok(());
        }

        let categories = Self::download().await?;

        println!("Initializing UI Categories Table");

        let id_map = categories.iter();
        for id_map in &id_map.chunks(BIND_MAX / 2) {
            QueryBuilder::new(SQL_INSERT)
                .push_values(id_map, |mut b, data| {
                    b.push_bind(data.id).push_bind(&data.name);
                })
                .build()
                .execute(self.db)
                .await?;
        }

        Ok(())
    }

    async fn download() -> Result<Vec<CsvUiCategory>> {
        println!("Downloading Items from Github");

        let reader = Cursor::new(download_file(CSV_FILE).await?);
        let mut categories = Vec::new();
        csv_parse!(reader => {
            id = U[0];
            name = S[1];
            categories.push(CsvUiCategory { id, name: name.to_string() });
        });

        Ok(categories)
    }
}

////////////////////////////////////////////////////////////

const SQL_TABLE_NAME: &str = "ui_categories";

const SQL_CREATE: &str = formatcp!(
    "CREATE TABLE IF NOT EXISTS {SQL_TABLE_NAME} (
        id          SMALLINT        UNSIGNED,
        name        VARCHAR(100)    NOT NULL,
        PRIMARY KEY     ( id )
    )"
);

const SQL_EMPTY: &str = formatcp!("SELECT COUNT(id) FROM {SQL_TABLE_NAME}");

const SQL_INSERT: &str = formatcp!("INSERT INTO {SQL_TABLE_NAME} (id, name) ");
