mod mock_server;

use anyhow::Result;
use async_processor::AsyncProcessor;
use ffxiv_universalis::{
    request_universalis_info, UniversalisHandleOutput, UniversalisHistory, UniversalisListing,
    UniversalisRequestType,
};
use futures::Future;
use tokio::runtime::Builder;

use mock_server::spawn_universalis_mock;

fn block<T, Fut: Future<Output = T>>(f: Fut) -> T {
    Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(f)
}

const ALL_IDS_ONE: [u32; 1] = [31980];
#[allow(dead_code)]
const ALL_IDS_SMALL: [u32; 10] = [9, 10, 5111, 5526, 12524, 12537, 31929, 31971, 31977, 31980];
#[allow(dead_code)]
const ALL_IDS_LARGE: [u32; 112] = [
    2, 4, 7, 8, 9, 10, 11, 12, 13, 4753, 4785, 4806, 4830, 4842, 4844, 4850, 5106, 5111, 5113,
    5143, 5232, 5261, 5262, 5263, 5346, 5350, 5372, 5389, 5485, 5491, 5516, 5518, 5522, 5526, 5728,
    7017, 12524, 12537, 12544, 12549, 12554, 12559, 12596, 12600, 12628, 12640, 12891, 19841,
    19844, 19846, 19847, 19848, 19856, 19864, 19865, 19868, 19869, 19871, 19872, 19873, 19877,
    19878, 19881, 19901, 19902, 19904, 19909, 19910, 19916, 19917, 19922, 19925, 19926, 19927,
    19930, 19931, 19932, 19938, 19939, 19940, 19941, 19942, 19943, 19944, 19945, 19950, 19951,
    19952, 19953, 19954, 19955, 19963, 19965, 19969, 19971, 19974, 19975, 19976, 19977, 19978,
    19979, 19980, 19981, 19982, 19983, 19984, 19985, 19986, 19993, 19994, 19995, 19996,
];

fn send_universalis_request<T: UniversalisRequestType>(
    async_processor: AsyncProcessor,
    all_ids: &[u32],
    worlds: Vec<String>,
) -> Result<UniversalisHandleOutput> {
    let all_ids = Vec::from(all_ids);
    Ok(block(async move {
        let (addr, ready, shutdown) = spawn_universalis_mock().unwrap();
        ready.await.unwrap();

        let addr = format!("http://{addr}");
        std::env::set_var("UNIVERSALIS_URL", addr);

        let mut universalis_handle =
            request_universalis_info::<T>(async_processor.clone(), worlds, all_ids, 7.0);
        universalis_handle.wait_for_ready().await;

        async_processor.disconnect();
        async_processor.await;
        shutdown();
        universalis_handle.await
    })?)
}

#[test]
fn test_listing() -> Result<()> {
    let (market_info, failures) = send_universalis_request::<UniversalisListing>(
        AsyncProcessor::new(1),
        &ALL_IDS_ONE,
        vec!["Dynamis".to_owned()],
    )?;

    for id in &ALL_IDS_ONE {
        assert!(market_info.contains_key(id));
    }
    assert!(failures.is_empty());

    Ok(())
}

#[test]
fn test_history() -> Result<()> {
    let (market_info, failures) = send_universalis_request::<UniversalisHistory>(
        AsyncProcessor::new(1),
        &ALL_IDS_ONE,
        vec!["Dynamis".to_owned()],
    )?;

    for id in &ALL_IDS_ONE {
        assert!(market_info.contains_key(id));
    }
    assert!(failures.is_empty());

    Ok(())
}
