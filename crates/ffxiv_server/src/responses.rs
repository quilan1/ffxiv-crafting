pub fn ok_json<T>(data: T) -> impl axum::response::IntoResponse
where
    (reqwest::StatusCode, axum::Json<T>): axum::response::IntoResponse,
{
    (reqwest::StatusCode::OK, axum::Json(data))
}

pub fn ok_text<S: Into<String>>(data: S) -> impl axum::response::IntoResponse
where
    (reqwest::StatusCode, String): axum::response::IntoResponse,
{
    (reqwest::StatusCode::OK, data.into())
}

pub fn not_found(data: String) -> impl axum::response::IntoResponse {
    (reqwest::StatusCode::NOT_FOUND, data)
}
