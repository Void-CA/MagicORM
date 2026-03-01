use crate::{meta::ModelMeta, relations::traits::HasFK};

use sqlx::{FromRow, sqlite::SqliteRow};

pub trait Model:
    ModelMeta
    + Sized
    + Send
    + Unpin
    + for<'r> FromRow<'r, SqliteRow>
{
    type Id: Send + std::fmt::Display;

    fn id(&self) -> &Self::Id;

    fn query<'a>() -> crate::query::QueryBuilder<'a, Self> {
        crate::query::QueryBuilder::new(Self::TABLE)
    }

    fn id_column() -> &'static str {
        "id"
    }
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