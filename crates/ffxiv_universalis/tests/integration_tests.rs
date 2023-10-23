mod mock_server;

mod mocks {
    use anyhow::Result;
    use ffxiv_universalis::{ListingsResults, Processor};

    use crate::mock_server::MockDownloader;

    const ALL_IDS_ONE: [u32; 1] = [31980];

    async fn send_universalis_request(
        processor: Processor,
        all_ids: &[u32],
        worlds: Vec<String>,
    ) -> Result<ListingsResults> {
        let all_ids = Vec::from(all_ids);

        let async_processor = processor.async_processor();
        let mut universalis_handle =
            processor.make_request::<MockDownloader>(&worlds, &all_ids, 7.0);

        async_processor.disconnect();
        async_processor.await;

        Ok(universalis_handle.collect_all().await)
    }

    #[tokio::test]
    async fn test_listings() -> Result<()> {
        let processor = Processor::new();
        let ListingsResults {
            listings,
            history,
            failures,
        } = send_universalis_request(processor, &ALL_IDS_ONE, vec!["Dynamis".to_owned()]).await?;

        for id in &ALL_IDS_ONE {
            assert!(listings.contains_key(id));
            assert!(history.contains_key(id));
        }
        assert!(failures.is_empty());

        Ok(())
    }
}
