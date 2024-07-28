use std::sync::Arc;

use axum::Router;
use futures::future::BoxFuture;
use sqlx::{pool::PoolConnection, Pool, SqlitePool, Transaction};

use crate::RDBMS;

mod response;
mod status;

struct Resources {
    pool: Pool<RDBMS>,
}

impl Resources {
    pub async fn with_bare_conn<F, T, E>(&self, callback: F) -> Result<T, E>
    where
        F: for<'c> FnOnce(
            &'c mut PoolConnection<RDBMS>,
        ) -> BoxFuture<'c, Result<T, E>>,
        E: From<sqlx::Error>,
    {
        let mut conn = self.pool.acquire().await?;
        let result = callback(&mut conn).await;
        conn.close().await?;
        result
    }

    pub async fn with_transaction<F, T, E>(&self, callback: F) -> Result<T, E>
    where
        F: for<'c> FnOnce(
            &'c mut Transaction<RDBMS>,
        ) -> BoxFuture<'c, Result<T, E>>,
        E: From<sqlx::Error>,
    {
        let mut transaction = self.pool.begin().await?;
        let result = callback(&mut transaction).await;
        if result.is_ok() {
            transaction.commit().await?;
        } else {
            transaction.rollback().await?;
        }
        result
    }
}

pub fn router(pool: SqlitePool) -> Router {
    let resources = Arc::new(Resources { pool });
    Router::new().nest("/status/", status::router(resources))
}
