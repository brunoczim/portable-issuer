use std::path::PathBuf;

use axum::{http::StatusCode, response::IntoResponse, routing::get, Router};
use sqlx::{Pool, Sqlite};

mod status;
mod api;
mod static_files;

pub type RDBMS = Sqlite;

pub fn router(static_path: impl Into<PathBuf>, pool: Pool<RDBMS>) -> Router {
    Router::new()
        .nest("/api/v1/", api::router(pool))
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
