use crate::{meta::ModelMeta, relations::traits::HasFK, traits::Model};

pub async fn load_has_many<P, C>(
    parent: &P,
    pool: &sqlx::SqlitePool,
) -> anyhow::Result<Vec<C>>
where
    P: Model,
    C: Model + ModelMeta + HasFK<P> + for<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> + Send + Unpin,
{
    let fk_column = C::fk_for_parent();
    let rows = C::query()
        .filter(fk_column, "=", parent.id())
        .fetch_all(pool)
        .await
        .map_err(|e| anyhow::anyhow!(e))?; // convertimos sqlx::Error → anyhow::Error

    Ok(rows)
}