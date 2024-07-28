use std::sync::Arc;

use axum::{
    extract::Path,
    http::StatusCode,
    routing::{delete, get, patch, post},
    Json,
    Router,
};
use futures::TryStreamExt;
use serde::{Deserialize, Serialize};
use sqlx::{query, Row};
use thiserror::Error;

use crate::status::ResponseStatusCode;

use super::{response::ApiResponse, Resources};

const NAME_UNIQUE_CONSTRAINT: &str = "un_issue_statuses_name";
const ISSUES_STATUS_FK: &str = "fk_issues_status";

#[derive(Debug, Clone, Deserialize)]
struct NewStatusPayload {
    name: String,
}

#[derive(Debug, Clone, Deserialize)]
struct PatchStatusPayload {
    #[serde(default)]
    name: Option<String>,
}

#[derive(Debug, Error)]
enum NewStatusError {
    #[error("Status with the given name already exists")]
    AlreadyExists,
    #[error("Failed to manipulate database resources")]
    Sqlx(#[source] sqlx::Error),
}

impl From<sqlx::Error> for NewStatusError {
    fn from(error: sqlx::Error) -> Self {
        if let sqlx::Error::Database(error) = &error {
            if error.is_unique_violation()
                && error.constraint() == Some(NAME_UNIQUE_CONSTRAINT)
            {
                return Self::AlreadyExists;
            }
        }
        Self::Sqlx(error)
    }
}

impl ResponseStatusCode for NewStatusError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::AlreadyExists => StatusCode::FORBIDDEN,
            Self::Sqlx(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[derive(Debug, Error)]
enum GetStatusError {
    #[error("Status not found")]
    NotFound,
    #[error("Failed to manipulate database resources")]
    Sqlx(#[source] sqlx::Error),
}

impl From<sqlx::Error> for GetStatusError {
    fn from(error: sqlx::Error) -> Self {
        if let sqlx::Error::RowNotFound = &error {
            return Self::NotFound;
        }
        Self::Sqlx(error)
    }
}

impl ResponseStatusCode for GetStatusError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Sqlx(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[derive(Debug, Error)]
enum DeleteStatusError {
    #[error("Status not found")]
    NotFound,
    #[error("Status cannot be deleted because it is in use")]
    InUse,
    #[error("Failed to manipulate database resources")]
    Sqlx(#[source] sqlx::Error),
}

impl From<sqlx::Error> for DeleteStatusError {
    fn from(error: sqlx::Error) -> Self {
        if let sqlx::Error::Database(error) = &error {
            if error.is_foreign_key_violation()
                && error.constraint() == Some(ISSUES_STATUS_FK)
            {
                return Self::InUse;
            }
        }
        if let sqlx::Error::RowNotFound = &error {
            return Self::NotFound;
        }
        Self::Sqlx(error)
    }
}

impl ResponseStatusCode for DeleteStatusError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::InUse => StatusCode::FORBIDDEN,
            Self::Sqlx(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[derive(Debug, Error)]
enum PatchStatusError {
    #[error("At least one field must be patched, none were")]
    NoFieldsPatched,
    #[error("Status with the given name already exists")]
    AlreadyExists,
    #[error("Status not found")]
    NotFound,
    #[error("Failed to manipulate database resources")]
    Sqlx(#[source] sqlx::Error),
}

impl From<sqlx::Error> for PatchStatusError {
    fn from(error: sqlx::Error) -> Self {
        if let sqlx::Error::Database(error) = &error {
            if error.is_unique_violation()
                && error.constraint() == Some(NAME_UNIQUE_CONSTRAINT)
            {
                return Self::AlreadyExists;
            }
        }
        if let sqlx::Error::RowNotFound = &error {
            return Self::NotFound;
        }
        Self::Sqlx(error)
    }
}

impl ResponseStatusCode for PatchStatusError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::NoFieldsPatched => StatusCode::BAD_REQUEST,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::AlreadyExists => StatusCode::FORBIDDEN,
            Self::Sqlx(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct StatusResponse {
    id: i64,
    name: String,
}

impl ResponseStatusCode for StatusResponse {
    fn status_code(&self) -> StatusCode {
        StatusCode::OK
    }
}

#[derive(Debug, Clone, Serialize)]
struct StatusListResponse {
    list: Vec<StatusResponse>,
}

impl ResponseStatusCode for StatusListResponse {
    fn status_code(&self) -> StatusCode {
        StatusCode::OK
    }
}

pub fn router(resources: Arc<Resources>) -> Router {
    Router::new()
        .route(
            "/new",
            post({
                let resources = resources.clone();
                move |body| post_new(body, resources)
            }),
        )
        .route(
            "/id/:id",
            get({
                let resources = resources.clone();
                move |id| get_by_id(id, resources)
            }),
        )
        .route(
            "/name/:name",
            get({
                let resources = resources.clone();
                move |name| get_by_name(name, resources)
            }),
        )
        .route(
            "/id/:id",
            delete({
                let resources = resources.clone();
                move |id| delete_by_id(id, resources)
            }),
        )
        .route(
            "/name/:name",
            delete({
                let resources = resources.clone();
                move |name| delete_by_name(name, resources)
            }),
        )
        .route(
            "/id/:id",
            patch({
                let resources = resources.clone();
                move |id, payload| patch_by_id(id, payload, resources)
            }),
        )
        .route(
            "/name/:name",
            patch({
                let resources = resources.clone();
                move |name, payload| patch_by_name(name, payload, resources)
            }),
        )
        .route(
            "/list/",
            get({
                let resources = resources.clone();
                move || get_list(resources)
            }),
        )
}

async fn post_new(
    Json(new_status): Json<NewStatusPayload>,
    resources: Arc<Resources>,
) -> ApiResponse<StatusResponse, NewStatusError> {
    resources
        .with_bare_conn(move |connection| {
            Box::pin(async move {
                let row = query(
                    "INSERT INTO statuses (name) VALUES (?) RETURNING id",
                )
                .bind(&new_status.name)
                .fetch_one(&mut **connection)
                .await?;
                let id = row.try_get("id")?;
                Ok(StatusResponse { id, name: new_status.name })
            })
        })
        .await
        .into()
}

async fn get_by_id(
    Path(id): Path<i64>,
    resources: Arc<Resources>,
) -> ApiResponse<StatusResponse, GetStatusError> {
    resources
        .with_bare_conn(|connection| {
            Box::pin(async move {
                let row = query("SELECT name FROM statuses WHERE id = ?")
                    .bind(&id)
                    .fetch_one(&mut **connection)
                    .await?;
                let name = row.try_get("name")?;
                Ok(StatusResponse { id, name })
            })
        })
        .await
        .into()
}

async fn get_by_name(
    Path(name): Path<String>,
    resources: Arc<Resources>,
) -> ApiResponse<StatusResponse, GetStatusError> {
    resources
        .with_bare_conn(|connection| {
            Box::pin(async move {
                let row = query("SELECT id FROM statuses WHERE name = ?")
                    .bind(&name)
                    .fetch_one(&mut **connection)
                    .await?;
                let id = row.try_get("id")?;
                Ok(StatusResponse { id, name })
            })
        })
        .await
        .into()
}

async fn delete_by_id(
    Path(id): Path<i64>,
    resources: Arc<Resources>,
) -> ApiResponse<StatusResponse, DeleteStatusError> {
    resources
        .with_bare_conn(|connection| {
            Box::pin(async move {
                let row =
                    query("DELETE FROM statuses WHERE id = ? RETURNING name")
                        .bind(&id)
                        .fetch_one(&mut **connection)
                        .await?;
                let name = row.try_get("name")?;
                Ok(StatusResponse { id, name })
            })
        })
        .await
        .into()
}

async fn delete_by_name(
    Path(name): Path<String>,
    resources: Arc<Resources>,
) -> ApiResponse<StatusResponse, DeleteStatusError> {
    resources
        .with_bare_conn(|connection| {
            Box::pin(async move {
                let row =
                    query("DELETE FROM statuses WHERE name = ? RETURNING id")
                        .bind(&name)
                        .fetch_one(&mut **connection)
                        .await?;
                let id = row.try_get("id")?;
                Ok(StatusResponse { id, name })
            })
        })
        .await
        .into()
}

async fn patch_by_id(
    Path(id): Path<i64>,
    Json(payload): Json<PatchStatusPayload>,
    resources: Arc<Resources>,
) -> ApiResponse<StatusResponse, PatchStatusError> {
    let Some(new_name) = payload.name else {
        return ApiResponse::new(Err(PatchStatusError::NoFieldsPatched));
    };
    resources
        .with_bare_conn(|connection| {
            Box::pin(async move {
                query("UPDATE statuses SET name = ? WHERE id = ?")
                    .bind(&new_name)
                    .bind(&id)
                    .execute(&mut **connection)
                    .await?;
                Ok(StatusResponse { id, name: new_name })
            })
        })
        .await
        .into()
}

async fn patch_by_name(
    Path(name): Path<String>,
    Json(payload): Json<PatchStatusPayload>,
    resources: Arc<Resources>,
) -> ApiResponse<StatusResponse, PatchStatusError> {
    let Some(new_name) = payload.name else {
        return ApiResponse::new(Err(PatchStatusError::NoFieldsPatched));
    };
    resources
        .with_bare_conn(|connection| {
            Box::pin(async move {
                let sql =
                    "UPDATE statuses SET name = ? WHERE name = ? RETURNING id";
                let row = query(sql)
                    .bind(&new_name)
                    .bind(&name)
                    .fetch_one(&mut **connection)
                    .await?;
                let id = row.try_get("id")?;
                Ok(StatusResponse { id, name: new_name })
            })
        })
        .await
        .into()
}

async fn get_list(
    resources: Arc<Resources>,
) -> ApiResponse<StatusListResponse, GetStatusError> {
    resources
        .with_bare_conn(|connection| {
            Box::pin(async move {
                let mut statuses = Vec::new();
                let mut stream =
                    query("SELECT id, name FROM statuses WHERE ORDER BY id")
                        .fetch(&mut **connection);
                while let Some(row) = stream.try_next().await? {
                    let id = row.try_get("id")?;
                    let name = row.try_get("name")?;
                    statuses.push(StatusResponse { id, name });
                }
                Ok(StatusListResponse { list: statuses })
            })
        })
        .await
        .into()
}
