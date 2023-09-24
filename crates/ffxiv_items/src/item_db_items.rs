use anyhow::Result;
use itertools::Itertools;

use crate::{ItemId, ItemInfo};

use super::{
    tables::{InputIdsTable, ItemInfoTable},
    ItemDB,
};

impl ItemDB {
    pub async fn ids_from_filters<S: AsRef<str>>(&self, filter_str: S) -> Result<Vec<u32>> {
        let mut top_ids = self.ids_from_filter_str(filter_str.as_ref()).await?;
        top_ids.sort();
        Ok(top_ids)
    }

    pub async fn associated_ids(&self, ids: &[u32]) -> Result<Vec<u32>> {
        let mut all_ids = InputIdsTable::new(self).by_item_ids(ids).await?;
        all_ids.extend(ids);

        let mut all_ids = all_ids.into_iter().unique().collect_vec();
        all_ids.sort();
        Ok(all_ids)
    }

    pub async fn items_from_ids<I: ItemId>(&self, ids: &[I]) -> Result<Vec<ItemInfo>> {
        ItemInfoTable::new(self).by_item_ids(ids).await
    }
}
