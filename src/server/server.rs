use anyhow::Result;
use axum::{http::Method, routing::get, Router};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};

use super::{custom::Custom, StaticFiles};

pub struct Server;

impl Server {
    pub async fn run() -> Result<()> {
        let app = Router::new()
            .route("/js/*path", get(StaticFiles::static_path))
            .route("/v1/custom-filter", get(Custom::custom_filter))
            .layer(
                CorsLayer::new()
                    .allow_methods(vec![Method::GET, Method::POST])
                    .allow_origin(Any),
            );

        println!("Server setup!");
        let addr = SocketAddr::from(([127, 0, 0, 1], 3001));
        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await?;

        Ok(())
    }
}
