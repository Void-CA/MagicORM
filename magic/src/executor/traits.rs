use async_trait::async_trait;

/// Abstracción sobre cualquier executor de SQL.
/// Implementado por los adaptadores en `executor::adapters`.
#[async_trait]
pub trait Executor {
    async fn execute(&self, sql: &str) -> anyhow::Result<()>;
}
