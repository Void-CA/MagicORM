use crate::model::{Model, ModelMeta};
use crate::relations::traits::HasFK;

pub async fn load_has_many<'e, P, C, E>(
    parent: &P,
    executor: E,
) -> anyhow::Result<Vec<C>>
where
    P: Model,
    P::Id: Copy,
    C: Model + ModelMeta + HasFK<P> + for<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> + Send + Unpin,
    E: sqlx::Executor<'e, Database = sqlx::Sqlite>,
{
    let fk_column = C::fk_for_parent();
    let rows = C::query()
        .filter(fk_column, "=", parent.id())
        .fetch_all(executor)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;

    Ok(rows)
}
