pub use tests::spawn_universalis_mock;

#[cfg(test)]
mod tests {
    use std::net::SocketAddr;

    use anyhow::Result;
    use axum::{extract::Path, http::Method, response::IntoResponse, routing::get, Router};
    use futures::channel::oneshot::{self, Receiver};
    use tokio::spawn;
    use tower_http::cors::{Any, CorsLayer};

    use ffxiv_universalis::json_types::{
        HistoryView, ItemListingView, ListingView, MultipleHistoryView, MultipleListingView,
    };

    pub fn spawn_universalis_mock() -> Result<(SocketAddr, Receiver<()>, impl FnOnce())> {
        let app = Router::new()
            .route("/api/v2/:world/:ids", get(get_listings))
            .route("/api/v2/history/:world/:ids", get(get_histories))
            .layer(
                CorsLayer::new()
                    .allow_methods(vec![Method::GET, Method::PUT])
                    .allow_headers(Any)
                    .allow_origin(Any),
            );

        let addr = SocketAddr::from(([127, 0, 0, 1], 0));
        let server = axum::Server::bind(&addr).serve(app.into_make_service());
        let addr = server.local_addr();

        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let (ready_tx, ready_rx) = oneshot::channel();
        let server = server.with_graceful_shutdown(async {
            shutdown_rx.await.ok();
        });

        let shutdown = || {
            let _ = shutdown_tx.send(());
        };

        spawn(async {
            ready_tx.send(()).unwrap();
            server.await.unwrap();
        });

        Ok((addr, ready_rx, shutdown))
    }

    async fn get_listings(Path((_world, ids)): Path<(String, String)>) -> impl IntoResponse {
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

        serde_json::to_string(&view).unwrap()
    }

    async fn get_histories(Path((world, ids)): Path<(String, String)>) -> impl IntoResponse {
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
                                    world_name: Some(world.clone()),
                                    retainer_name: None,
                                })
                                .collect(),
                        },
                    )
                })
                .collect(),
        };

        serde_json::to_string(&view).unwrap()
    }
}
