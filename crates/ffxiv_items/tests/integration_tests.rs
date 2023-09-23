#[cfg(feature = "docker")]
mod docker {
    use anyhow::Result;
    use ffxiv_items::ItemDB;

    async fn database() -> Result<ItemDB> {
        let item_db_conn = std::env::var("FFXIV_ITEM_DB_CONN").unwrap();
        ItemDB::connect(item_db_conn).await
    }

    #[tokio::test]
    async fn test_filter_name() -> Result<()> {
        let db = database().await?;
        let (top_ids, all_ids) = db.get_ids_from_filters(":name Eagle Feather").await?;

        const ITEM_ID: u32 = 5358;
        assert_eq!(top_ids, vec![ITEM_ID]);
        assert_eq!(all_ids, vec![ITEM_ID]);

        let items = db.items_from_ids(&all_ids).await?;
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id, ITEM_ID);
        assert_eq!(items[0].name, "Eagle Feather");
        assert!(items[0].recipe.is_none());

        Ok(())
    }
}
