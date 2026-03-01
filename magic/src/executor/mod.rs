pub mod error;
pub use error::SchemaError;
use async_trait::async_trait;

#[async_trait]
pub trait Executor {
    type Error;

    async fn execute(&self, sql: &str) -> Result<(), Self::Error>;
}

#[cfg(feature = "sqlite")]
#[async_trait::async_trait]
impl Executor for sqlx::SqlitePool {
    type Error = sqlx::Error;

    async fn execute(&self, sql: &str) -> Result<(), Self::Error> {
        sqlx::query(sql).execute(self).await?;
        Ok(())
    }
}