// Central DB type alias — the ONLY place where feature flags decide the default database.
// This simulates an associated type default at the crate level (stable Rust).
// All other modules use `T::DB` or `crate::db::DefaultDB`, never `#[cfg]` directly.

#[cfg(feature = "postgres")]
/// Use PostgreSQL when the `postgres` feature is active.
pub type DefaultDB = sqlx::Postgres;

#[cfg(not(any(feature = "postgres", feature = "sqlite")))]
compile_error!("Either 'sqlite' or 'postgres' feature must be enabled");

#[cfg(all(not(feature = "postgres"), feature = "sqlite"))]
/// Default to SQLite when the `sqlite` feature is active.
pub type DefaultDB = sqlx::Sqlite;
