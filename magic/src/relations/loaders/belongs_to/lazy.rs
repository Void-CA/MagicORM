use std::time::Instant;

use crate::model::Model;
use tracing::debug;

/// Carga lazy del padre por FK.
pub async fn load_belongs_to<'e, P>(
    executor: impl sqlx::Executor<'e, Database = P::DB>,
    id: <P as Model>::Id,
) -> anyhow::Result<P>
where
    P: Model,
    for<'q> <P::DB as sqlx::Database>::Arguments<'q>: sqlx::IntoArguments<'q, P::DB>,
{
    let start = Instant::now();
    let sql = format!("SELECT * FROM {} WHERE id = ?", P::TABLE);
    debug!(table = P::TABLE, sql = %sql, "load_belongs_to");

    let result = sqlx::query_as::<_, P>(&sql)
        .bind(id)
        .fetch_one(executor)
        .await;
    let elapsed = start.elapsed();

    match result {
        Ok(row) => {
            debug!(elapsed_us = elapsed.as_micros() as u64, "load_belongs_to done");
            Ok(row)
        }
        Err(e) => {
            debug!(error = %e, elapsed_us = elapsed.as_micros() as u64, "load_belongs_to failed");
            Err(anyhow::anyhow!(e))
        }
    }
}
