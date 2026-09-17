//! # MagicORM
//!
//! A Rust ORM for SQL databases with a focus on ease of use, low boilerplate,
//! and performance. Built for fast-evolving systems where models, fields, and
//! relationships change constantly.
//!
//! - **Backends:** SQLite (default) and PostgreSQL
//! - **Schema:** automatic table creation, introspection, and migrations
//! - **Queries:** typed [`QueryBuilder`](crate::query), filters, joins, eager loading
//! - **Scale:** batch insertion, streaming, keyset pagination, retention helpers
//!
//! ## Quickstart
//!
//! ```no_run
//! use magic_orm::{prelude::*, register_models};
//!
//! #[derive(MagicModel, Debug)]
//! #[magic(table = "users")]
//! pub struct User {
//!     pub id: i64,
//!     pub name: String,
//! }
//!
//! register_models!(User);
//!
//! # async fn example() -> anyhow::Result<()> {
//! let pool = Sqlite::pool("app.db").await?;
//! create_all::<_, AppModels>(&pool).await?;
//!
//! let id = User::insert(&pool, &User::new("Alicia".into())).await?;
//! let user = User::get_by_id(&pool, id).await?.unwrap();
//! assert_eq!(user.name, "Alicia");
//! # Ok(())
//! # }
//! ```
//!
//! See the repository README and `docs/` for guides and architecture decisions.

pub mod crud; // Helpers CRUD compartidos multi-backend
pub mod db; // Central DB type alias (DefaultDB)
pub mod describe; // Describe trait + helpers de serialización de metadatos
pub mod dialect; // SqlDialect trait + implementaciones por backend
pub mod model; // módulo unificado: ModelMeta, Model, HasMany, ModelDescriptor, register_models!
pub mod prelude;
pub mod query;
pub mod relations;
pub mod schema;

#[cfg(feature = "sqlite")]
pub mod sqlite;

pub use magic_derive::MagicModel;
