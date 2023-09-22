use anyhow::Result;
use axum::{
    http::Method,
    routing::{get, put},
    Router,
};
use ffxiv_items::ItemDB;
use futures::join;
use std::{net::SocketAddr, sync::Arc};
use tower_http::cors::{Any, CorsLayer};

use crate::{
    market::{self, MarketState},
    recipe,
};

pub struct Server;

#[allow(unused_must_use)]
impl Server {
    pub async fn run(db: ItemDB) -> Result<()> {
        let market_state = MarketState::new();
        let async_processor = market_state.async_processor();
        let db = Arc::new(db);

        let health_service = Router::new().route("/health", get(|| async { "OK" }));

        let recipe_service = Router::new()
            .route("/recipe", get(recipe::get_recipe_info))
            .layer(axum_server_timing::ServerTimingLayer::new("RecipeService"))
            .with_state(db.clone());

        let market_service = Router::new()
            .nest(
                "/market",
                Router::new()
                    .route("/history", put(market::put_market_history))
                    .route("/listings", put(market::put_market_listings))
                    .route("/:id", get(market::get_market_info))
                    .route("/:id/cancel", put(market::put_market_cancel)),
            )
            .layer(axum_server_timing::ServerTimingLayer::new("MarketService"))
            .with_state((market_state.clone(), db.clone()));

        let v1_router = Router::new()
            .merge(recipe_service)
            .merge(market_service)
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
            async_processor,
            axum::Server::bind(&addr).serve(app.into_make_service())
        );

        Ok(())
    }
}
