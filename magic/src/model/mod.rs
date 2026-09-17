// Submódulos internos — los detalles de implementación permanecen ocultos
mod core;
mod descriptor;
mod meta;
mod registry; // contiene la macro register_models!

// ---------------------------------------------------------------------------
// API pública del módulo unificado
// Los consumidores del crate solo necesitan importar desde `model::*`
// ---------------------------------------------------------------------------

pub use core::{BelongsTo, HasMany, Model};
pub use descriptor::{ModelDescriptor, RegisteredModels, SchemaDescriptor};
pub use meta::{ColumnMeta, ForeignKeyMeta, IndexMeta, ModelMeta};
#[allow(unused_imports)]
pub use registry::*; // re-exporta la macro register_models!
