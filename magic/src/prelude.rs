// Core model traits
pub use crate::traits::Model;
pub use crate::traits::model::HasMany;
pub use crate::relations::traits::HasFK;

// Model meta and registration
pub use crate::meta::ModelMeta;
pub use crate::register_models;

// Macros and helpers
pub use crate::has_many;
pub use crate::MagicModel;

// Schema utilities
pub use crate::schema::create_all;
pub use crate::schema::create::create_table_sql;

// External dependencies
pub use anyhow::Error;
pub use paste::paste;
pub use sqlx::SqlitePool;
