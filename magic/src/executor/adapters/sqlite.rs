use async_trait::async_trait;
use sqlx::SqlitePool;
use crate::executor::traits::Executor;

/// Adaptador de SQLite: implementa [`Executor`] sobre un pool de conexiones.
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
