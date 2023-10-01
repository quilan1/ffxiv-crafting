use std::time::Instant;

use anyhow::Result;
use const_format::formatcp;
use futures::TryStreamExt;
use sqlx::Row;

use crate::{
    tables::{strip_whitespace, ItemInfoTable},
    Filter, FilterBindingInfo, ItemDB,
};

impl ItemDB {
    pub(crate) async fn ids_from_filter_str(&self, filter_str: &str) -> Result<Vec<u32>> {
        let start = Instant::now();
        let Some(FilterBindingInfo { clause, binds }) = Filter::from_filter_str(filter_str) else {
            return Ok(Vec::new());
        };

        let query_string = strip_whitespace(format!("{SQL_SELECT} WHERE {clause}"));
        let mut sql_query = sqlx::query(&query_string);
        for bind in binds {
            sql_query = sql_query.bind(bind);
        }

        let mut ids = Vec::new();
        let mut sql_query = sql_query.persistent(true).fetch(self);
        while let Some(row) = sql_query.try_next().await? {
            ids.push(row.get::<u32, _>(0));
        }

        log::debug!(target: "ffxiv_items", "Query for filter string ({} ids returned): {:.3}s", ids.len(), start.elapsed().as_secs_f32());
        Ok(ids)
    }
}

const SQL_SELECT: &str = formatcp!(
    "SELECT i.id, i.name FROM {} AS i",
    ItemInfoTable::SQL_TABLE_NAME
);
