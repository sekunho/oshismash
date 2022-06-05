use deadpool_postgres::{
    BuildError, Manager, ManagerConfig, Object, Pool, PoolError, RecyclingMethod,
};
use tokio_postgres::NoTls;

pub struct Handle {
    pub pool: Pool,
}

impl Handle {
    // TODO: Consume an `AppConfig`
    pub fn from_config() -> Result<Handle, BuildError> {
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

    pub async fn get_client(&self) -> Result<Object, PoolError> {
        let client = self.pool.get().await?;

        Ok(client)
    }
}
