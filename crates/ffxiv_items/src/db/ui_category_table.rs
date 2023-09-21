use anyhow::Result;
use const_format::formatcp;
use itertools::Itertools;
use sqlx::QueryBuilder;

use crate::parsers;

use super::{ItemDB, BIND_MAX};

////////////////////////////////////////////////////////////

impl_table!(UiCategoryTable);

impl UiCategoryTable<'_> {
    pub async fn initialize(&self, categories: &parsers::UiCategoryList) -> Result<()> {
        if !self.is_empty().await? {
            return Ok(());
        }

        println!("Initializing UI Categories Table");

        let id_map = categories.categories.iter();
        for id_map in &id_map.chunks(BIND_MAX / 2) {
            QueryBuilder::new(SQL_INSERT)
                .push_values(id_map, |mut b, (&id, name)| {
                    b.push_bind(id).push_bind(name);
                })
                .build()
                .execute(self.db)
                .await?;
        }

        Ok(())
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
