use anyhow::Result;
use axum::{http::Method, routing::get, Router};
use futures::join;
use parking_lot::Mutex;
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tower_http::cors::{Any, CorsLayer};

use crate::util::AsyncProcessor;

use super::{
    custom::{Custom, CustomLazyInfo},
    StaticFiles,
};

pub struct Server;

#[derive(Clone)]
pub struct ServerState {
    pub async_processor: AsyncProcessor,
    pub async_general_processor: AsyncProcessor,
    pub lazy_records: Arc<Mutex<HashMap<String, CustomLazyInfo>>>,
}

#[allow(unused_must_use)]
impl Server {
    pub async fn run() -> Result<()> {
        let async_processor = AsyncProcessor::new(8);
        let async_general_processor = AsyncProcessor::new(10000);

        let app_state = Arc::new(ServerState {
            async_processor: async_processor.clone(),
            async_general_processor: async_general_processor.clone(),
            lazy_records: Arc::new(Mutex::new(HashMap::new())),
        });

        let app = Router::with_state(app_state)
            .route("/web/*path", get(StaticFiles::static_path))
            .route("/v1/custom-filter", get(Custom::custom_filter))
            .route("/v1/custom", get(Custom::get_lazy).put(Custom::put_lazy))
            .layer(
                CorsLayer::new()
                    .allow_methods(vec![Method::GET, Method::POST])
                    .allow_origin(Any),
            );

        let addr = SocketAddr::from(([127, 0, 0, 1], 3001));
        println!("Server up at http://127.0.0.1:3001/web/harness.html");

        join!(
            async_processor,
            async_general_processor,
            axum::Server::bind(&addr).serve(app.into_make_service())
        );

        Ok(())
    }
}
