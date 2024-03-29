#[cfg(feature = "docker")]
mod docker {
    use std::collections::HashSet;

    use anyhow::Result;
    use ffxiv_items::ItemDB;

    async fn database() -> Result<ItemDB> {
        let item_db_conn = std::env::var("FFXIV_ITEM_DB_CONN").unwrap();
        ItemDB::connect(item_db_conn).await
    }

    #[tokio::test]
    async fn test_filter_empty() -> Result<()> {
        let db = database().await?;
        let ids = db.ids_from_query("").await?;
        assert!(ids.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn test_filter_name() -> Result<()> {
        let db = database().await?;
        let ids = db.ids_from_query(":name Eagle Feather").await?;

        const ITEM_ID: u32 = 5358;
        assert_eq!(ids, vec![ITEM_ID]);

        let items = db.items_from_ids(&ids).await?;
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id, ITEM_ID);
        assert_eq!(items[0].name, "Eagle Feather");
        assert!(items[0].recipe.is_none());

        Ok(())
    }

    #[tokio::test]
    async fn test_filter_name_exact() -> Result<()> {
        let db = database().await?;
        let ids = db.ids_from_query(":name !Eagle Feather").await?;

        const ITEM_ID: u32 = 5358;
        assert_eq!(ids, vec![ITEM_ID]);

        let items = db.items_from_ids(&ids).await?;
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id, ITEM_ID);
        assert_eq!(items[0].name, "Eagle Feather");
        assert!(items[0].recipe.is_none());

        Ok(())
    }

    #[tokio::test]
    async fn test_filter_name_exact_many() -> Result<()> {
        let db = database().await?;
        let ids = db
            .ids_from_query(":name !(Eagle Feather|Maple Branch)")
            .await?;
        // Eagle Feather, Maple Branch
        assert_eq!(ids, vec![5358, 5396]);

        Ok(())
    }

    #[tokio::test]
    async fn test_filter_name_regex() -> Result<()> {
        let db = database().await?;
        let ids = db.ids_from_query(":name (Mind|Strength) Alkahest").await?;
        let items = db.items_from_ids(&ids).await?;

        for item in items {
            let ends_with_mind = item.name.ends_with("Mind Alkahest");
            let ends_with_strength = item.name.ends_with("Strength Alkahest");
            assert!(ends_with_mind || ends_with_strength);
            assert!(item.recipe.is_some());
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_filter_rlevel_empty() -> Result<()> {
        let db = database().await?;
        let ids = db.ids_from_query(":rlevel").await?;
        assert!(ids.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn test_filter_rlevel() -> Result<()> {
        let db = database().await?;
        let ids = db.ids_from_query(":rlevel 90, :name Mind Alkahest").await?;
        let items = db.items_from_ids(&ids).await?;

        for item in items {
            assert!(item.name.ends_with("Mind Alkahest"));
            assert!(item.recipe.is_some());
            assert_eq!(item.recipe.unwrap().level, 90);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_filter_rlevel_range() -> Result<()> {
        let db = database().await?;
        let ids = db
            .ids_from_query(":rlevel 80|90, :name Mind Alkahest")
            .await?;
        let items = db.items_from_ids(&ids).await?;

        let mut seen_levels = HashSet::new();
        for item in items {
            assert!(item.name.ends_with("Mind Alkahest"));
            assert!(item.recipe.is_some());

            let level = item.recipe.unwrap().level;
            assert!(level >= 80);
            assert!(level <= 90);
            seen_levels.insert(level);
        }
        assert!(seen_levels.len() > 1);

        Ok(())
    }

    #[tokio::test]
    async fn test_filter_elevel_empty() -> Result<()> {
        let db = database().await?;
        let ids = db.ids_from_query(":elevel").await?;
        assert!(ids.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn test_filter_elevel() -> Result<()> {
        let db = database().await?;
        let ids = db.ids_from_query(":elevel 90, :name of Ascension").await?;
        let items = db.items_from_ids(&ids).await?;

        for item in items {
            assert!(item.name.ends_with("of Ascension"));
            assert!(item.recipe.is_none());
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_filter_elevel_range() -> Result<()> {
        let db = database().await?;
        let ids = db.ids_from_query(":elevel 86|88, :name of healing").await?;
        let items = db.items_from_ids(&ids).await?;
        assert!(items
            .iter()
            .any(|item| item.name.starts_with("Blue Zircon"))); // Level 86
        assert!(items.iter().any(|item| item.name.starts_with("Ktiseos"))); // Level 87
        assert!(items
            .iter()
            .any(|item| item.name.starts_with("Star Quartz"))); // Level 88
        Ok(())
    }

    #[tokio::test]
    async fn test_filter_ilevel_empty() -> Result<()> {
        let db = database().await?;
        let ids = db.ids_from_query(":ilevel").await?;
        assert!(ids.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn test_filter_ilevel() -> Result<()> {
        let db = database().await?;
        let ids = db.ids_from_query(":ilevel 155").await?;
        // Althyk Lavender, Voidrake
        assert_eq!(ids, vec![15857, 15858]);

        Ok(())
    }

    #[tokio::test]
    async fn test_filter_ilevel_range() -> Result<()> {
        let db = database().await?;
        let ids = db
            .ids_from_query(":ilevel 136|139, :name of healing")
            .await?;
        let items = db.items_from_ids(&ids).await?;
        assert!(items.iter().any(|item| item.name.starts_with("Orthodox"))); // Level 136
        assert!(items.iter().any(|item| item.name.starts_with("Hallowed"))); // Level 139
        Ok(())
    }

    #[tokio::test]
    async fn test_filter_ui_category_empty() -> Result<()> {
        let db = database().await?;
        let ids = db.ids_from_query(":cat").await?;
        assert!(ids.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn test_filter_ui_category() -> Result<()> {
        let db = database().await?;
        let ids = db.ids_from_query(":cat Metal, :ilevel 2").await?;
        // Bronze Rings, Bronze Rivets
        assert_eq!(ids, vec![5081, 5091]);
        Ok(())
    }

    #[tokio::test]
    async fn test_filter_ui_category_range() -> Result<()> {
        let db = database().await?;
        let ids = db.ids_from_query(":cat Metal|Lumber, :ilevel 2").await?;
        // Bronze Rings, Bronze Rivets, Maple Branch
        assert_eq!(ids, vec![5081, 5091, 5396]);
        Ok(())
    }

    #[tokio::test]
    async fn test_filter_contains_empty() -> Result<()> {
        let db = database().await?;
        let ids = db.ids_from_query(":contains").await?;
        assert_eq!(ids.len(), 0);
        Ok(())
    }

    #[tokio::test]
    async fn test_filter_contains() -> Result<()> {
        let db = database().await?;
        let ids = db.ids_from_query(":contains maple branch").await?;
        // Maple Longbow, Plumed Maple Shortbow, Maple Wand, Maple Fishing Rod
        assert_eq!(ids, vec![1892, 1893, 1958, 2572]);
        Ok(())
    }

    #[tokio::test]
    async fn test_filter_includes_empty() -> Result<()> {
        let db = database().await?;
        let ids = db.ids_from_query(":includes").await?;
        assert_eq!(ids.len(), 0);
        Ok(())
    }

    #[tokio::test]
    async fn test_filter_includes() -> Result<()> {
        let db = database().await?;
        let ids = db.ids_from_query(":includes !maple branch").await?;
        // Maple Longbow, Plumed Maple Shortbow, Wrapped Maple Longbow, Wrapped Elm Longbow,
        // Maple Wand, Whispering Maple Wand, Budding Maple Wand, Maple Fishing Rod
        assert_eq!(ids, vec![1892, 1893, 1894, 1905, 1958, 1959, 1960, 2572]);
        Ok(())
    }

    #[tokio::test]
    async fn test_filter_or_clauses_ilevel() -> Result<()> {
        let db = database().await?;
        let ids = db.ids_from_query(":ilevel 155; :ilevel 51").await?;
        // Dzemael Tomato Seeds, Honey Lemon Seeds, Prickly Pineapple Seeds,
        // Althyk Lavender, Voidrake
        assert_eq!(ids, vec![7724, 7733, 7734, 15857, 15858]);
        Ok(())
    }

    #[tokio::test]
    async fn test_filter_or_clauses_name() -> Result<()> {
        let db = database().await?;
        let ids = db
            .ids_from_query(":name !maple branch; :name !eagle feather")
            .await?;
        // Eagle Feather, Maple Branch
        assert_eq!(ids, vec![5358, 5396]);
        Ok(())
    }

    #[tokio::test]
    async fn test_filter_or_clauses_includes_name() -> Result<()> {
        let db = database().await?;
        let ids = db
            .ids_from_query(":name !(maple branch|eagle feather); :includes !maple branch")
            .await?;
        // Maple Longbow, Plumed Maple Shortbow, Wrapped Maple Longbow, Wrapped Elm Longbow,
        // Maple Wand, Whispering Maple Wand, Budding Maple Wand, Maple Fishing Rod,
        // Eagle Feather, Maple Branch
        assert_eq!(
            ids,
            vec![1892, 1893, 1894, 1905, 1958, 1959, 1960, 2572, 5358, 5396]
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_filter_or_clauses_category_rlevel() -> Result<()> {
        let db = database().await?;
        let ids = db
            .ids_from_query(":cat Metal|Lumber, :ilevel 2; :rlevel 80, :name Mind Alkahest")
            .await?;
        // Bronze Rings, Bronze Rivets, Maple Branch,
        // Grade [2..4] Mind Alkahest
        assert_eq!(ids, vec![5081, 5091, 5396, 27795, 29967, 32943]);
        Ok(())
    }
}
