use std::io::Cursor;

use anyhow::Result;
use const_format::formatcp;
use itertools::Itertools;
use sqlx::QueryBuilder;

use crate::ItemDB;

use super::{download_file, strip_whitespace, BIND_MAX};

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
        let categories = Self::download().await?;

        println!("Initializing UI Categories Table");
        let id_map = categories.iter();
        for id_map in &id_map.chunks(BIND_MAX / 2) {
            QueryBuilder::new(strip_whitespace(SQL_INSERT))
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
        println!("Downloading UI Categories from Github");

        let reader = Cursor::new(download_file(CSV_FILE).await?);
        let mut categories = Vec::new();
        csv_parse!(reader => {
            id = U[0];
            name = S[1];
            if id == 0 || name.is_empty() {
                continue;
            }

            categories.push(CsvUiCategory { id, name: name.to_string() });
        });

        Ok(categories)
    }
}

////////////////////////////////////////////////////////////

const SQL_TABLE_NAME: &str = "ui_categories";

const SQL_CREATE: &str = formatcp!(
    "CREATE TABLE IF NOT EXISTS {SQL_TABLE_NAME} (
        id          SMALLINT        UNSIGNED    PRIMARY KEY,
        name        VARCHAR(50)     NOT NULL,
        INDEX       ( name )
    )"
);

const SQL_INSERT: &str = formatcp!("INSERT INTO {SQL_TABLE_NAME} (id, name) ");
