use sqlx::{Database, IntoArguments};
use crate::model::Model;

/// Carga lazy del padre por FK.
pub async fn load_belongs_to<'e, P>(
    executor: impl sqlx::Executor<'e, Database = P::DB>,
    id: <P as Model>::Id,
) -> anyhow::Result<P>
where
    P: Model,
    for<'q> <P::DB as Database>::Arguments<'q>: IntoArguments<'q, P::DB>,
{
    let sql = format!("SELECT * FROM {} WHERE id = ?", P::TABLE);
    sqlx::query_as::<_, P>(&sql)
        .bind(id)
        .fetch_one(executor)
        .await
        .map_err(|e| anyhow::anyhow!(e))
}
