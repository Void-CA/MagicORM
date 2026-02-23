use crate::{meta::ModelMeta, traits::Model};

pub async fn load_has_many<P, C>(
    parent: &P,
    pool: &sqlx::SqlitePool,
    fk_column: &str
) -> sqlx::Result<Vec<C>>
where
    P: Model,
    C: Model + ModelMeta + for<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> + Send + Unpin,
{
    C::query()
        .filter(fk_column, "=", parent.id())
        .fetch_all(pool)
        .await
}