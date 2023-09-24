#[cfg(feature = "docker")]
mod mock_server;

#[cfg(feature = "docker")]
mod docker {
    mod mocks {
        use anyhow::Result;
        use ffxiv_universalis::{
            UniversalisHandleOutput, UniversalisHistory, UniversalisListing, UniversalisProcessor,
            UniversalisRequestType,
        };

        use crate::mock_server::spawn_universalis_mock;

        const ALL_IDS_ONE: [u32; 1] = [31980];

        async fn send_universalis_request<T: UniversalisRequestType>(
            processor: UniversalisProcessor,
            all_ids: &[u32],
            worlds: Vec<String>,
        ) -> Result<UniversalisHandleOutput> {
            let all_ids = Vec::from(all_ids);

            let (addr, ready, shutdown) = spawn_universalis_mock().unwrap();
            ready.await.unwrap();

            let addr = format!("http://{addr}");
            std::env::set_var("UNIVERSALIS_URL", addr);

            let async_processor = processor.async_processor();
            let mut universalis_handle = processor.make_request::<T>(worlds, all_ids, 7.0);
            universalis_handle.wait_for_ready().await;

            async_processor.disconnect();
            async_processor.await;
            shutdown();

            Ok(universalis_handle.await?)
        }

        #[tokio::test]
        async fn test_listing() -> Result<()> {
            let processor = UniversalisProcessor::new();
            let (market_info, failures) = send_universalis_request::<UniversalisListing>(
                processor,
                &ALL_IDS_ONE,
                vec!["Dynamis".to_owned()],
            )
            .await?;

            for id in &ALL_IDS_ONE {
                assert!(market_info.contains_key(id));
            }
            assert!(failures.is_empty());

            Ok(())
        }

        #[tokio::test]
        async fn test_history() -> Result<()> {
            let processor = UniversalisProcessor::new();
            let (market_info, failures) = send_universalis_request::<UniversalisHistory>(
                processor,
                &ALL_IDS_ONE,
                vec!["Dynamis".to_owned()],
            )
            .await?;

            for id in &ALL_IDS_ONE {
                assert!(market_info.contains_key(id));
            }
            assert!(failures.is_empty());

            Ok(())
        }
    }
}
