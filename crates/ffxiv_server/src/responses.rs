use axum::{
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

pub trait JsonResponse
where
    Self: Sized + Serialize,
    Json<Self>: IntoResponse,
{
    fn ok(self) -> Response {
        (reqwest::StatusCode::OK, Json(self)).into_response()
    }

    fn not_found(self) -> Response {
        (reqwest::StatusCode::NOT_FOUND, Json(self)).into_response()
    }
}

pub trait StringResponse
where
    Self: Sized,
    Self: Into<String>,
{
    fn ok(self) -> Response {
        (reqwest::StatusCode::OK, self.into()).into_response()
    }

    fn not_found(self) -> Response {
        (reqwest::StatusCode::NOT_FOUND, self.into()).into_response()
    }

    fn server_error(self) -> Response {
        (reqwest::StatusCode::NOT_FOUND, self.into()).into_response()
    }
}

impl StringResponse for String {}
impl StringResponse for &str {}
