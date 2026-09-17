pub mod mysql;
pub mod postgres;
pub mod sqlite;

pub use postgres::PostgresDialect;
pub use sqlite::SqliteDialect;

// ---------------------------------------------------------------------------
// SqlDialect — abstrae las diferencias de sintaxis entre backends SQL.
// Cada backend implementa este trait como struct cero-cost (ZST).
// ---------------------------------------------------------------------------
pub trait SqlDialect: Send + Sync + 'static {
    /// Placeholder para parámetros posicionales: "?" para SQLite, "$1" para Postgres.
    fn placeholder(index: usize) -> String;

    /// Delimita un identificador (tabla/columna) según las reglas del dialecto.
    fn quote_identifier(name: &str) -> String;

    /// Genera la sentencia INSERT con returning.
    /// Para SQLite: "INSERT INTO t (c1, c2) VALUES (?, ?)"
    /// Para Postgres: "INSERT INTO t (c1, c2) VALUES ($1, $2) RETURNING id"
    fn insert_returning(table: &str, cols: &[&str], pk: &str) -> String;

    /// Genera la sentencia UPSERT (INSERT ... ON CONFLICT DO UPDATE).
    /// Conflicto siempre en la PK; actualiza todas las columnas no-PK.
    /// Para SQLite: "INSERT INTO t (c1,c2) VALUES (?,?) ON CONFLICT("id") DO UPDATE SET "c1"=excluded."c1", "c2"=excluded."c2""
    /// Para Postgres: misma sentencia + RETURNING "id"
    fn upsert_returning(table: &str, cols: &[&str], pk: &str) -> String;

    /// Expresión para obtener el último ID insertado.
    /// Some("last_insert_rowid()") para SQLite, None para Postgres (usa RETURNING).
    fn last_insert_id_expr() -> Option<&'static str>;

    /// Sentencia para habilitar FK enforcement si el backend lo requiere.
    /// Some("PRAGMA foreign_keys = ON;") para SQLite, None para Postgres.
    fn enable_foreign_keys() -> Option<&'static str>;

    /// Mapea un tipo de Rust a su representación SQL en este dialecto.
    fn map_rust_type(rust_ty: &str) -> &'static str;

    /// Genera SQL para eliminar una foreign key.
    /// Postgres: ALTER TABLE t DROP CONSTRAINT <name>;
    /// SQLite: secuencia de table rebuild (PRAGMA off, CREATE new, INSERT SELECT, DROP old, RENAME).
    fn drop_foreign_key(
        table: &str,
        fk: &crate::model::ForeignKeyMeta,
        columns: &[crate::model::ColumnMeta],
        foreign_keys: &[crate::model::ForeignKeyMeta],
    ) -> String;
}

// ---------------------------------------------------------------------------
// HasDialect — asocia un backend sqlx::Database con su SqlDialect.
// ---------------------------------------------------------------------------
pub trait HasDialect {
    type Dialect: SqlDialect;
}

#[cfg(feature = "sqlite")]
impl HasDialect for sqlx::Sqlite {
    type Dialect = SqliteDialect;
}

#[cfg(feature = "postgres")]
impl HasDialect for sqlx::Postgres {
    type Dialect = PostgresDialect;
}
