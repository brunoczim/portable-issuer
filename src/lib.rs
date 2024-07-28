use std::path::PathBuf;

use axum::{
    http::{HeaderMap, HeaderName, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};

mod status;
mod api;
mod static_files;

pub fn router(static_path: impl Into<PathBuf>) -> Router {
    Router::new()
        .nest("/api/v1/", api::router())
        .nest("/static/", static_files::router(static_path))
        .route("/", get(get_root))
}

async fn get_root() -> impl IntoResponse {
    (
        StatusCode::PERMANENT_REDIRECT,
        [("Location", "/static/index.html")],
        "Permanent redirect",
    )
}
