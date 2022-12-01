use anyhow::Result;
use axum::{http::Method, routing::get, Router};
use futures::join;
use std::{net::SocketAddr, sync::Arc};
use tower_http::cors::{Any, CorsLayer};

use crate::universalis::{UniversalisListingAsyncProcessor, UniversalisRequestAsyncProcessor};

use super::{custom::Custom, StaticFiles};

pub struct Server;

#[derive(Clone)]
pub struct ServerState<'a> {
    pub listing_processor: UniversalisListingAsyncProcessor<'a>,
    pub request_processor: UniversalisRequestAsyncProcessor<'a>,
}

#[allow(unused_must_use)]
impl Server {
    pub async fn run() -> Result<()> {
        let listing_processor = UniversalisListingAsyncProcessor::new("Listings", 8);
        let request_processor = UniversalisRequestAsyncProcessor::new("Request", 1000);

        let app_state = Arc::new(ServerState {
            listing_processor: listing_processor.clone(),
            request_processor: request_processor.clone(),
        });
        let app = Router::with_state(app_state)
            .route("/web/*path", get(StaticFiles::static_path))
            .route("/v1/custom-filter", get(Custom::custom_filter))
            .layer(
                CorsLayer::new()
                    .allow_methods(vec![Method::GET, Method::POST])
                    .allow_origin(Any),
            );

        let addr = SocketAddr::from(([127, 0, 0, 1], 3001));
        println!("Server setup at http://127.0.0.1:3001");

        join!(
            listing_processor,
            request_processor,
            axum::Server::bind(&addr).serve(app.into_make_service())
        );

        Ok(())
    }
}
