use std::time::Instant;

use anyhow::Result;
use const_format::formatcp;
use futures::TryStreamExt;
use sqlx::Row;

use crate::{
    tables::{strip_whitespace, ItemInfoTable},
    ItemDB, Query, QueryBindingInfo,
};

impl ItemDB {
    /// Returns items that match a particular query string.
    pub async fn ids_from_query<S: AsRef<str>>(&self, query: S) -> Result<Vec<u32>> {
        let start = Instant::now();
        let Some(QueryBindingInfo { clause, binds }) = Query::from_query(query.as_ref()) else {
            return Ok(Vec::new());
        };

        let db_query_string = strip_whitespace(format!("{SQL_SELECT} WHERE {clause}"));
        let mut sql_query = sqlx::query(&db_query_string);
        for bind in binds {
            sql_query = sql_query.bind(bind);
        }

        let mut ids = Vec::new();
        let mut sql_query = sql_query.persistent(true).fetch(self);
        while let Some(row) = sql_query.try_next().await? {
            ids.push(row.get::<u32, _>(0));
        }

        log::debug!(target: "ffxiv_items", "DB Query for query string ({} ids returned): {:.3}s", ids.len(), start.elapsed().as_secs_f32());
        ids.sort();
        Ok(ids)
    }
}

const SQL_SELECT: &str = formatcp!(
    "SELECT i.id, i.name FROM {} AS i",
    ItemInfoTable::SQL_TABLE_NAME
);
