use anyhow::Result;

use crate::{ItemId, ItemInfo};

use super::{
    tables::{InputIdsTable, ItemInfoTable},
    ItemDB,
};

impl ItemDB {
    pub async fn get_ids_from_filters<S: AsRef<str>>(
        &self,
        filter_str: S,
    ) -> Result<(Vec<u32>, Vec<u32>)> {
        let top_ids = self.ids_from_filter_str(filter_str.as_ref()).await?;
        let all_ids = InputIdsTable::new(self).by_item_ids(&top_ids).await?;
        Ok((top_ids, all_ids))
    }

    pub async fn items_from_ids<I: ItemId>(&self, ids: &[I]) -> Result<Vec<ItemInfo>> {
        ItemInfoTable::new(self).by_item_ids(ids).await
    }

    pub async fn item<I: ItemId>(&self, id: I) -> Result<Option<ItemInfo>> {
        Ok(ItemInfoTable::new(self)
            .by_item_ids(&[id])
            .await?
            .first()
            .cloned())
    }
}
