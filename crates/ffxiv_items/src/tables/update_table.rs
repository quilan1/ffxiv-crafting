use std::time::Instant;

use anyhow::{bail, Result};
use chrono::{DateTime, FixedOffset};
use const_format::formatcp;
use futures::TryStreamExt;
use sqlx::{QueryBuilder, Row};

use crate::ItemDB;

use super::{impl_table, strip_whitespace};

impl_table!(UpdateTable);

impl UpdateTable<'_> {
    pub async fn last_updated(&self) -> Result<DateTime<FixedOffset>> {
        let start = Instant::now();
        let query_string = strip_whitespace(SQL_SELECT);

        let mut sql_query = sqlx::query(&query_string).persistent(true).fetch(self.db);
        let Some(row) = sql_query.try_next().await? else {
            bail!("Couldn't find last updated date");
        };
        let last_updated = row.try_get::<String, _>(0)?;
        log::debug!(target: "ffxiv_items", "Query for updates: {:.3}s", start.elapsed().as_secs_f32());

        Ok(DateTime::parse_from_rfc3339(&last_updated)?)
    }

    pub async fn insert(&self, last_updated: &DateTime<FixedOffset>) -> Result<()> {
        println!("Initializing Update Table");
        let last_updated = vec![last_updated.to_rfc3339()];
        QueryBuilder::new(strip_whitespace(SQL_INSERT))
            .push_values(last_updated, |mut b, item| {
                b.push_bind(item);
            })
            .build()
            .execute(self.db)
            .await?;

        Ok(())
    }

    pub async fn update(&self, last_updated: &DateTime<FixedOffset>) -> Result<()> {
        println!("Updating Update Table");
        let last_updated = last_updated.to_rfc3339();
        QueryBuilder::new(strip_whitespace(SQL_UPDATE))
            .push_bind(last_updated.to_string())
            .build()
            .execute(self.db)
            .await?;

        Ok(())
    }
}

const SQL_TABLE_NAME: &str = "updates";

const SQL_CREATE: &str =
    formatcp!("CREATE TABLE IF NOT EXISTS {SQL_TABLE_NAME} ( date VARCHAR(40) PRIMARY KEY )");

const SQL_INSERT: &str = formatcp!("INSERT INTO {SQL_TABLE_NAME} (date)");

const SQL_UPDATE: &str = formatcp!("UPDATE {SQL_TABLE_NAME} SET date = ");

const SQL_SELECT: &str = formatcp!("SELECT date FROM {SQL_TABLE_NAME}");
