use anyhow::Result;
use async_processor::{AmValue, AsyncProcessor};
use axum::{http::Method, routing::get, Router};
use futures::join;
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tower_http::cors::{Any, CorsLayer};

use crate::{
    general_listing::{get_item_history, get_item_listings, put_item_history, put_item_listings},
    recipe::get_recipe_info,
};

use crate::ListingInfo;

pub struct Server;

#[derive(Clone)]
pub struct ServerState {
    pub universalis_async_processor: AsyncProcessor,
    pub listings: AmValue<HashMap<String, ListingInfo>>,
}

#[allow(unused_must_use)]
impl Server {
    pub async fn run() -> Result<()> {
        let universalis_async_processor = AsyncProcessor::new(8);

        let app_state = Arc::new(ServerState {
            universalis_async_processor: universalis_async_processor.clone(),
            listings: AmValue::new(HashMap::new()),
        });

        let health_service = Router::new().route("/health", get(|| async { "OK" }));

        let recipe_service = Router::new()
            .route("/recipe", get(get_recipe_info))
            .layer(axum_server_timing::ServerTimingLayer::new("RecipeService"));

        let listing_service = Router::new()
            .route("/history", get(get_item_history).put(put_item_history))
            .route("/listings", get(get_item_listings).put(put_item_listings))
            .layer(axum_server_timing::ServerTimingLayer::new("ListingService"))
            .with_state(app_state);

        let v1_router = Router::new()
            .merge(recipe_service)
            .merge(listing_service)
            .merge(health_service);

        let app = Router::new().nest("/v1", v1_router).layer(
            CorsLayer::new()
                .allow_methods(vec![Method::GET, Method::PUT])
                .allow_headers(Any)
                .allow_origin(Any),
        );

        let addr = SocketAddr::from(([0, 0, 0, 0], 3001));
        println!("Server up at http://localhost:3001/");

        join!(
            universalis_async_processor,
            axum::Server::bind(&addr).serve(app.into_make_service())
        );

        Ok(())
    }
}
