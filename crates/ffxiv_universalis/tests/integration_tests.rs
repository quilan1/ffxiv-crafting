mod mock_server;

mod mocks {
    use anyhow::Result;
    use ffxiv_universalis::{Processor, ProcessorHandleOutput};

    use crate::mock_server::MockDownloader;

    const ALL_IDS_ONE: [u32; 1] = [31980];

    async fn send_universalis_request(
        processor: Processor,
        all_ids: &[u32],
        worlds: Vec<String>,
    ) -> Result<ProcessorHandleOutput> {
        let all_ids = Vec::from(all_ids);

        let async_processor = processor.async_processor();
        let mut universalis_handle =
            processor.make_request::<MockDownloader>(&worlds, &all_ids, 7.0);
        universalis_handle.wait_for_ready().await;

        async_processor.disconnect();
        async_processor.await;

        Ok(universalis_handle.await?)
    }

    #[tokio::test]
    async fn test_listings() -> Result<()> {
        let processor = Processor::new();
        let ProcessorHandleOutput {
            listings,
            history,
            failure_ids,
        } = send_universalis_request(processor, &ALL_IDS_ONE, vec!["Dynamis".to_owned()]).await?;

        for id in &ALL_IDS_ONE {
            assert!(listings.contains_key(id));
            assert!(history.contains_key(id));
        }
        assert!(failure_ids.is_empty());

        Ok(())
    }
}
