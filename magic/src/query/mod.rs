mod builder;
mod executor; // impl block de QueryBuilder con fetch_all, build_sql, etc.
mod eager;

pub use builder::{QueryBuilder};
pub use eager::EagerQueryBuilder; // QueryBuilder especializado para relaciones HasMany con carga eager