// Core model traits y tipos
pub use crate::model::Model;
pub use crate::model::HasMany;
pub use crate::model::BelongsTo;
pub use crate::model::ModelMeta;
pub use crate::model::ModelDescriptor;
pub use crate::model::RegisteredModels;
pub use crate::relations::traits::HasFK;

// Macros
pub use crate::register_models;
pub use crate::has_many;
pub use crate::MagicModel;

// Schema utilities
pub use crate::schema::create_all;
pub use crate::schema::create::create_table_sql;

// External dependencies re-exportados
pub use anyhow::Error;
pub use paste::paste;
pub use sqlx::SqlitePool;
