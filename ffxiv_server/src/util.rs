use crate::cli::settings;

pub fn make_builder(data_center: Option<String>) -> ffxiv_universalis::UniversalisBuilder {
    let settings = settings();
    let builder =
        ffxiv_universalis::UniversalisBuilder::new(&settings.homeworld, &settings.data_centers);
    match data_center {
        None => builder,
        Some(data_center) => builder.data_centers(data_center.split(',').collect::<Vec<_>>()),
    }
}

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
