use sqlx::{Database, IntoArguments};
use crate::model::Model;
use crate::relations::traits::HasFK;

/// Carga lazy de hijos para un padre.
pub async fn load_has_many<'e, P, C>(
    parent: &P,
    executor: impl sqlx::Executor<'e, Database = P::DB>,
) -> anyhow::Result<Vec<C>>
where
    P: Model,
    C: Model<DB = P::DB> + HasFK<P>,
    for<'q> <P::DB as Database>::Arguments<'q>: IntoArguments<'q, P::DB>,
{
    let fk_column = C::fk_for_parent();
    let sql = format!("SELECT * FROM {} WHERE {} = ?", C::TABLE, fk_column);
    sqlx::query_as::<_, C>(&sql)
        .bind(parent.id().clone())
        .fetch_all(executor)
        .await
        .map_err(|e| anyhow::anyhow!(e))
}
