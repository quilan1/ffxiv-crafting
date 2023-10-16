use anyhow::Result;
use ffxiv_universalis::json_types::{
    HistoryView, ItemListingView, ListingView, MultipleHistoryView, MultipleListingView,
};
use futures::FutureExt;
use mock_traits::FileDownloader;

pub struct MockDownloader {}

impl MockDownloader {
    async fn get_listings(ids: &str) -> Result<String> {
        let ids = ids.split(',').map(ToString::to_string).collect::<Vec<_>>();
        let view = MultipleListingView {
            items: ids
                .into_iter()
                .map(|id| {
                    (
                        id,
                        ListingView {
                            listings: (0..2)
                                .map(|_| ItemListingView {
                                    price_per_unit: 0,
                                    hq: false,
                                    quantity: 1,
                                    last_review_time: Some(0),
                                    timestamp: None,
                                    world_name: None,
                                    retainer_name: Some("Retainer".into()),
                                })
                                .collect(),
                        },
                    )
                })
                .collect(),
        };

        Ok(serde_json::to_string(&view)?)
    }

    async fn get_histories(world: &str, ids: &str) -> Result<String> {
        let ids = ids.split(',').map(ToString::to_string).collect::<Vec<_>>();
        let view = MultipleHistoryView {
            items: ids
                .into_iter()
                .map(|id| {
                    (
                        id,
                        HistoryView {
                            entries: (0..2)
                                .map(|_| ItemListingView {
                                    price_per_unit: 0,
                                    hq: false,
                                    quantity: 1,
                                    last_review_time: None,
                                    timestamp: Some(0),
                                    world_name: Some(world.to_string()),
                                    retainer_name: None,
                                })
                                .collect(),
                        },
                    )
                })
                .collect(),
        };

        Ok(serde_json::to_string(&view)?)
    }
}

impl FileDownloader for MockDownloader {
    fn download(url: &str) -> futures::future::BoxFuture<'_, Result<String>> {
        match url {
            "https://universalis.app/api/v2/Dynamis/31980,2?entries=0" => {
                Self::get_listings("31980,2").boxed()
            }
            "https://universalis.app/api/v2/history/Dynamis/31980,2" => {
                Self::get_histories("Dynamis", "31980,2").boxed()
            }
            _ => panic!("No match for {url}"),
        }
    }
}
