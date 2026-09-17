// Core model traits y tipos
pub use crate::describe::Describe;
pub use crate::model::BelongsTo;
pub use crate::model::HasMany;
pub use crate::model::Model;
pub use crate::model::ModelDescriptor;
pub use crate::model::ModelMeta;
pub use crate::model::RegisteredModels;
pub use crate::model::SchemaDescriptor;
pub use crate::relations::traits::HasFK;

// Macros
pub use crate::MagicModel;
pub use crate::has_many;
pub use crate::register_models;

// Schema utilities
pub use crate::schema::create::create_table_sql;
pub use crate::schema::create_all;

// External dependencies re-exportados
pub use anyhow::Error;
pub use paste::paste;

#[cfg(feature = "sqlite")]
pub use crate::sqlite::{Sqlite, SqliteConfig};

#[cfg(feature = "sqlite")]
pub use sqlx::SqlitePool;
