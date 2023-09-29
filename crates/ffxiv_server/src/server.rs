use anyhow::Result;
use axum::{http::Method, routing::get, Router};
use ffxiv_items::ItemDB;
use ffxiv_universalis::UniversalisProcessor;
use futures::join;
use std::{net::SocketAddr, sync::Arc};
use tower_http::cors::{Any, CorsLayer};

use crate::universalis::universalis_websocket;

pub struct Server;

#[allow(unused_must_use)]
impl Server {
    pub async fn run(db: ItemDB) -> Result<()> {
        let universalis_processor = UniversalisProcessor::new();
        let async_processor = universalis_processor.async_processor();
        let db = Arc::new(db);

        let health_service = Router::new().route("/health", get(|| async { "OK" }));

        let market_service_ws = Router::new()
            .route("/universalis", get(universalis_websocket))
            .with_state((universalis_processor.clone(), db.clone()));

        let v1_router = Router::new().merge(health_service).merge(market_service_ws);

        let app = Router::new().nest("/v1", v1_router).layer(
            CorsLayer::new()
                .allow_methods(vec![Method::GET, Method::PUT])
                .allow_headers(Any)
                .allow_origin(Any),
        );

        let addr = SocketAddr::from(([0, 0, 0, 0], 3001));
        println!("Server up at http://localhost:3001/");

        join!(
            async_processor,
            axum::Server::bind(&addr).serve(app.into_make_service())
        );

        Ok(())
    }
}
