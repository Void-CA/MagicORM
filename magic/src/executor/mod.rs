// executor.rs
use async_trait::async_trait;
use sqlx::SqlitePool;

#[async_trait]
pub trait Executor {
    async fn execute(&self, sql: &str) -> anyhow::Result<()>;
}

#[async_trait]
impl Executor for SqlitePool {
    async fn execute(&self, sql: &str) -> anyhow::Result<()> {
        sqlx::query(sql)
            .execute(self)
            .await
            .map_err(|e| anyhow::anyhow!(e))?;
        Ok(())
    }
}