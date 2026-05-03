use crate::model::{Model, ModelMeta};

pub async fn load_belongs_to<'e, R, E>(
    executor: E,
    id: R::Id,
) -> anyhow::Result<R>
where
    R: Model + ModelMeta + for<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> + Send + Unpin,
    R::Id: for<'q> sqlx::Encode<'q, sqlx::Sqlite> + sqlx::Type<sqlx::Sqlite>,
    E: sqlx::Executor<'e, Database = sqlx::Sqlite>,
{
    let sql = format!("SELECT * FROM {} WHERE id = ?", R::TABLE);
    let row = sqlx::query_as::<_, R>(&sql)
        .bind(id)
        .fetch_one(executor)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;
    Ok(row)
}