use std::path::Path;

use axum::{
    body::{Empty, Full},
    extract::Path as ExtPath,
    http::HeaderValue,
    response::{IntoResponse, Response},
};
use reqwest::{header::CONTENT_TYPE, StatusCode};
use tokio::{fs::File, io::AsyncReadExt};

pub struct StaticFiles;

impl StaticFiles {
    pub async fn static_path(ExtPath(path): ExtPath<String>) -> impl IntoResponse {
        let path = path.trim_start_matches('/');
        let mime_type = mime_guess::from_path(path).first_or_text_plain();

        let path = Path::new("./src-web").join(path);
        println!("GET static {path:?}");

        match Path::exists(path.as_ref()) {
            false => Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(axum::body::boxed(Empty::new()))
                .unwrap(),
            true => {
                let mut file = File::open(path).await.unwrap();
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer).await.unwrap();

                Response::builder()
                    .status(StatusCode::OK)
                    .header(
                        CONTENT_TYPE,
                        HeaderValue::from_str(mime_type.as_ref()).unwrap(),
                    )
                    .body(axum::body::boxed(Full::from(buffer)))
                    .unwrap()
            }
        }
    }
}
