use axum::response::{IntoResponse, Response};
use deadpool_postgres::{Manager, ManagerConfig, Object, Pool, RecyclingMethod};
use hyper::StatusCode;
use tokio_postgres::NoTls;

pub struct Handle {
    pub pool: Pool,
}

#[derive(Debug)]
pub enum Error {
    FailedToBuildPool(deadpool_postgres::BuildError),
    FailedToGetClient(deadpool_postgres::PoolError),
}

impl From<deadpool_postgres::BuildError> for Error {
    fn from(e: deadpool_postgres::BuildError) -> Self {
        Error::FailedToBuildPool(e)
    }
}

impl From<deadpool_postgres::PoolError> for Error {
    fn from(e: deadpool_postgres::PoolError) -> Self {
        Error::FailedToGetClient(e)
    }
}

impl IntoResponse for Error {
    fn into_response(self: Error) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "500 INTERNAL SERVER ERROR",
        )
            .into_response()
    }
}

impl Handle {
    // TODO: Consume an `AppConfig`
    pub fn from_config() -> Result<Handle, Error> {
        let mut pg_config = tokio_postgres::Config::new();

        pg_config
            .dbname("oshismash-db")
            .host("localhost")
            .user("sekun")
            .port(5432);

        let manager_config = ManagerConfig {
            recycling_method: RecyclingMethod::Fast,
        };

        let manager = Manager::from_config(pg_config, NoTls, manager_config);

        let pool = Pool::builder(manager).max_size(22).build()?;

        Ok(Handle { pool })
    }

    pub async fn get_client(&self) -> Result<Object, Error> {
        let client = self.pool.get().await?;

        Ok(client)
    }
}
