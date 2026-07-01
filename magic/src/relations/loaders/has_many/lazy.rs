use std::time::Instant;

use crate::model::Model;
use crate::relations::traits::HasFK;
use tracing::debug;

/// Carga lazy de hijos para un padre.
pub async fn load_has_many<'e, P, C>(
    parent: &P,
    executor: impl sqlx::Executor<'e, Database = P::DB>,
) -> anyhow::Result<Vec<C>>
where
    P: Model,
    C: Model<DB = P::DB> + HasFK<P>,
    for<'q> <P::DB as sqlx::Database>::Arguments<'q>: sqlx::IntoArguments<'q, P::DB>,
{
    let start = Instant::now();
    let fk_column = C::fk_for_parent();
    let sql = format!("SELECT * FROM {} WHERE {} = ?", C::TABLE, fk_column);
    debug!(table = C::TABLE, fk_column, sql = %sql, "load_has_many");

    let result = sqlx::query_as::<_, C>(&sql)
        .bind(parent.id().clone())
        .fetch_all(executor)
        .await;
    let elapsed = start.elapsed();

    match result {
        Ok(rows) => {
            debug!(count = rows.len(), elapsed_us = elapsed.as_micros() as u64, "load_has_many done");
            Ok(rows)
        }
        Err(e) => {
            debug!(error = %e, elapsed_us = elapsed.as_micros() as u64, "load_has_many failed");
            Err(anyhow::anyhow!(e))
        }
    }
}
