use crate::model::meta::ModelMeta;
use crate::relations::traits::HasFK;

// ---------------------------------------------------------------------------
// Model — trait central que todo modelo derivado implementa
// ---------------------------------------------------------------------------

pub trait Model:
    ModelMeta
    + Sized
    + Send
    + Unpin
    + for<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow>
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

// ---------------------------------------------------------------------------
// BelongsTo — relación N:1 (el hijo conoce al padre por FK)
// ---------------------------------------------------------------------------

pub trait BelongsTo<P: Model>: Model {
    fn foreign_key() -> &'static str;
}

// ---------------------------------------------------------------------------
// HasMany — relación 1:N con carga lazy desde la base de datos
// ---------------------------------------------------------------------------

#[async_trait::async_trait]
pub trait HasMany<C>: Model
where
    Self::Id: Copy,
    C: Model
        + ModelMeta
        + HasFK<Self>
        + Send
        + Unpin
        + for<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow>,
{
    async fn load_children<'e, E>(&self, executor: E) -> anyhow::Result<Vec<C>>
    where
        E: sqlx::Executor<'e, Database = sqlx::Sqlite>,
    {
        crate::relations::load_has_many::<Self, C, E>(self, executor).await
    }
}