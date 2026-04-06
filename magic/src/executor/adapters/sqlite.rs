use async_trait::async_trait;
use sqlx::{SqlitePool, Sqlite, Transaction};
use crate::executor::traits::Executor;

#[async_trait]
impl Executor for SqlitePool {
    async fn execute(&mut self, sql: &str) -> anyhow::Result<()> {
        sqlx::query(sql)
            .execute(&*self) 
            .await?;
        Ok(())
    }
}

#[async_trait]
impl<'a> Executor for Transaction<'a, Sqlite> {
    async fn execute(&mut self, sql: &str) -> anyhow::Result<()> {
        let conn: &mut sqlx::SqliteConnection = &mut *self;

        sqlx::query(sql)
            .execute(conn)
            .await?;

        Ok(())
    }
}