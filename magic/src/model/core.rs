use crate::dialect::HasDialect;
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
    + for<'r> sqlx::FromRow<'r, <Self::DB as sqlx::Database>::Row>
{
    type Id: Send
        + std::fmt::Display
        + Clone
        + Eq
        + std::hash::Hash
        + for<'q> sqlx::Encode<'q, Self::DB>
        + sqlx::Type<Self::DB>;
    type DB: sqlx::Database + HasDialect;

    fn id(&self) -> &Self::Id;

    fn query<'a>() -> crate::query::QueryBuilder<'a, Self::DB, Self> {
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
    C: Model<DB = Self::DB>
        + ModelMeta
        + HasFK<Self>
        + Send
        + Unpin,
{
    async fn load_children<'e, E>(&self, executor: E) -> anyhow::Result<Vec<C>>
    where
        E: sqlx::Executor<'e, Database = Self::DB>,
        for<'q> <Self::DB as sqlx::Database>::Arguments<'q>: sqlx::IntoArguments<'q, Self::DB>,
    {
        crate::relations::load_has_many(self, executor).await
    }
}