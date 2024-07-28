use std::{
    path::{Component, Path, PathBuf},
    sync::Arc,
};

use axum::{
    body::{Body, Bytes},
    extract,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use futures::Stream;
use thiserror::Error;
use tokio::{
    fs::File,
    io::{self, BufReader},
};
use tokio_util::io::ReaderStream;

#[derive(Debug, Error)]
enum RequestError {
    #[error("Failed to open file")]
    FileOpen(#[source] io::Error),
    #[error("Given sub-path is invalid")]
    InvalidSubPath(String),
}

impl IntoResponse for RequestError {
    fn into_response(self) -> Response {
        match self {
            Self::FileOpen(error) => match error.kind() {
                io::ErrorKind::NotFound => {
                    (StatusCode::NOT_FOUND, "Not found").into_response()
                },
                _ => {
                    (StatusCode::UNPROCESSABLE_ENTITY, "Unprocessable content")
                        .into_response()
                },
            },
            Self::InvalidSubPath(_) => {
                (StatusCode::BAD_REQUEST, "Bad request").into_response()
            },
        }
    }
}

#[derive(Debug)]
struct Resources {
    base_dir: PathBuf,
}

impl Resources {
    async fn stream_file(
        &self,
        subpath: String,
    ) -> Result<
        impl Stream<Item = Result<Bytes, io::Error>> + Send + 'static,
        RequestError,
    > {
        let full_path = self.base_dir.join(&subpath);
        if Path::new(&subpath)
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
        {
            return Err(RequestError::InvalidSubPath(subpath));
        }
        let file =
            File::open(&full_path).await.map_err(RequestError::FileOpen)?;
        let reader = ReaderStream::new(BufReader::new(file));
        Ok(reader)
    }
}

pub fn router(base_dir: impl Into<PathBuf>) -> Router {
    let resources = Arc::new(Resources { base_dir: base_dir.into() });
    Router::new().route(
        "/*path",
        get(move |extract::Path(subpath)| async move {
            match resources.stream_file(subpath).await {
                Ok(stream) => {
                    (StatusCode::OK, Body::from_stream(stream)).into_response()
                },
                Err(error) => error.into_response(),
            }
        }),
    )
}
