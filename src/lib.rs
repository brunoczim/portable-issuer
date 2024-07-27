use std::path::PathBuf;

use axum::Router;

mod status;
mod api;
mod static_files;

pub fn router(static_path: impl Into<PathBuf>) -> Router {
    Router::new()
        .nest("/api/v1/", api::router())
        .nest("/static/", static_files::router(static_path))
}
