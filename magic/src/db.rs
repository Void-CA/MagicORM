// Central DB type alias — the ONLY place where feature flags decide the default database.
// This simulates an associated type default at the crate level (stable Rust).
// All other modules use `T::DB` or `crate::db::DefaultDB`, never `#[cfg]` directly.

#[cfg(feature = "postgres")]
/// Use PostgreSQL when the `postgres` feature is active.
pub type DefaultDB = sqlx::Postgres;

#[cfg(not(feature = "postgres"))]
/// Default to SQLite when the `postgres` feature is NOT active.
pub type DefaultDB = sqlx::Sqlite;
