mod mock_server;

mod mocks {
    use anyhow::Result;
    use ffxiv_universalis::{ListingsResults, Processor, RequestBuilder};

    use crate::mock_server::MockDownloader;

    const ALL_IDS_ONE: [u32; 1] = [31980];

    async fn send_universalis_request(
        processor: Processor,
        all_ids: &[u32],
        purchase_from: String,
    ) -> Result<ListingsResults> {
        let all_ids = Vec::from(all_ids);

        let async_processor = processor.async_processor();
        let mut universalis_handle = RequestBuilder::new(&all_ids, purchase_from)
            .retain_num_days(7.0)
            .execute::<MockDownloader>(&processor);

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
        } = send_universalis_request(processor, &ALL_IDS_ONE, "Dynamis".to_owned()).await?;

        for id in &ALL_IDS_ONE {
            assert!(listings.contains_key(id));
            assert!(history.contains_key(id));
        }
        assert!(failures.is_empty());

        Ok(())
    }
}
