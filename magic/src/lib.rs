pub mod db;      // Central DB type alias (DefaultDB)
pub mod dialect; // SqlDialect trait + implementaciones por backend
pub mod model;   // módulo unificado: ModelMeta, Model, HasMany, ModelDescriptor, register_models!
pub mod query;
pub mod relations;
pub mod schema;
pub mod executor;
pub mod prelude;

pub use magic_derive::MagicModel;
