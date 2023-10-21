use anyhow::Result;
use itertools::Itertools;

use crate::{ItemId, ItemInfo};

use super::{
    tables::{InputIdsTable, ItemInfoTable},
    ItemDB,
};

impl ItemDB {
    /// Returns all descendant ids of any of the `ids` that are recipes.
    pub async fn associated_ids(&self, ids: &[u32]) -> Result<Vec<u32>> {
        let mut all_ids = InputIdsTable::new(self).by_item_ids(ids).await?;
        all_ids.extend(ids);

        let mut all_ids = all_ids.into_iter().unique().collect_vec();
        all_ids.sort();
        Ok(all_ids)
    }

    /// Returns [ItemInfo] for each of the `ids` passed in.
    pub async fn items_from_ids<I: ItemId>(&self, ids: &[I]) -> Result<Vec<ItemInfo>> {
        ItemInfoTable::new(self).by_item_ids(ids).await
    }

    /// Returns top-level ids, descendant ids and [ItemInfo] data for an input
    /// query string.
    pub async fn all_info_from_query<S: AsRef<str>>(
        &self,
        query: S,
    ) -> Result<(Vec<u32>, Vec<u32>, Vec<ItemInfo>)> {
        let top_ids = self.ids_from_query(query).await?;
        let all_ids = self.associated_ids(&top_ids).await?;
        let items = self.items_from_ids(&all_ids).await?;
        Ok((top_ids, all_ids, items))
    }
}
