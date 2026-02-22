use crate::meta::ModelMeta;

pub async fn load_belongs_to<R>(
    pool: &sqlx::SqlitePool,
    id: i64,
) -> sqlx::Result<R>
where
    R: ModelMeta + for<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow>+ Send + Unpin,
{
    let sql = format!(
        "SELECT * FROM {} WHERE id = ?",
        R::TABLE
    );

    sqlx::query_as::<_, R>(&sql)
        .bind(id)
        .fetch_one(pool)
        .await
}