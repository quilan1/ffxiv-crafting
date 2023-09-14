use anyhow::Result;
use async_processor::{AmValue, AsyncProcessor};
use axum::{http::Method, routing::get, Router};
use futures::join;
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tower_http::cors::{Any, CorsLayer};

use crate::{
    gen_listing::{get_item_history, get_item_listings, put_item_history, put_item_listings},
    recipe::get_recipe_info,
};

use crate::ListingInfo;

pub struct Server;

#[derive(Clone)]
pub struct ServerState {
    pub async_processor: AsyncProcessor,
    pub listings: AmValue<HashMap<String, ListingInfo>>,
}

#[allow(unused_must_use)]
impl Server {
    pub async fn run() -> Result<()> {
        let async_processor = AsyncProcessor::new(8);

        let app_state = Arc::new(ServerState {
            async_processor: async_processor.clone(),
            listings: AmValue::new(HashMap::new()),
        });

        let app = Router::new()
            .route("/v1/recipe", get(get_recipe_info))
            .route("/v1/history", get(get_item_history).put(put_item_history))
            .route(
                "/v1/listings",
                get(get_item_listings).put(put_item_listings),
            )
            .with_state(app_state)
            .route("/v1/health", get(get_health_check))
            .layer(
                CorsLayer::new()
                    .allow_methods(vec![Method::GET, Method::PUT])
                    .allow_headers(Any)
                    .allow_origin(Any),
            );

        let addr = SocketAddr::from(([0, 0, 0, 0], 3001));
        println!("Server up at http://127.0.0.1:3001/");

        join!(
            async_processor,
            axum::Server::bind(&addr).serve(app.into_make_service())
        );

        Ok(())
    }
}

#[allow(clippy::unused_async)]
#[axum_macros::debug_handler]
pub async fn get_health_check() -> impl axum::response::IntoResponse {
    "OK"
}
