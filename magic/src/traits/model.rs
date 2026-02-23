use async_trait::async_trait;

use crate::meta::ModelMeta;

pub trait Model: Sized + Send {
    type Id: Send + std::fmt::Display;

    fn table_name() -> &'static str;
    fn columns() -> &'static [&'static str];
    fn id_column() -> &'static str {
        "id"
    }

    fn query<'a>() -> crate::query::QueryBuilder<'a, Self>;
    fn id(&self) -> &Self::Id;
}

pub trait BelongsTo<P: Model>: Model {
    fn foreign_key() -> &'static str;
}

#[async_trait]
pub trait HasMany<C>: Model
where
    C: BelongsTo<Self> + ModelMeta + Send + Unpin + for<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow>,
{
    async fn load_children<'a>(
        &self,
        pool: &'a sqlx::SqlitePool,
        fk_column: &str,
    ) -> sqlx::Result<Vec<C>> {
        crate::relations::load_has_many::<Self, C>(self, pool, fk_column).await
    }
}