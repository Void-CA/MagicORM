use crate::{meta::ModelMeta, relations::traits::HasFK};

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

#[async_trait::async_trait]
pub trait HasMany<C>: Model
where
    C: Model + ModelMeta + HasFK<Self> + Send + Unpin + for<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow>,
{
    async fn load_children<'a>(&self, pool: &'a sqlx::SqlitePool) -> anyhow::Result<Vec<C>> {
        crate::relations::runtime::has_many::load_has_many::<Self, C>(self, pool).await
    }
}