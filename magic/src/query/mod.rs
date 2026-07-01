mod builder;
mod executor; // impl block de QueryBuilder con fetch_all, build_sql, etc.
mod eager;
pub mod statement; // Statement<DB> + BindArg

pub use builder::QueryBuilder;
pub use eager::EagerQueryBuilder; // QueryBuilder especializado para relaciones HasMany con carga eager
pub use statement::{Statement, BindArg};