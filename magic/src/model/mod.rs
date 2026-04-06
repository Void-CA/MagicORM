// Submódulos internos — los detalles de implementación permanecen ocultos
mod meta;
mod core;
mod descriptor;
mod registry; // contiene la macro register_models!

// ---------------------------------------------------------------------------
// API pública del módulo unificado
// Los consumidores del crate solo necesitan importar desde `model::*`
// ---------------------------------------------------------------------------

pub use meta::{ColumnMeta, ForeignKeyMeta, ModelMeta};
pub use core::{Model, HasMany, BelongsTo};
pub use descriptor::{ModelDescriptor, RegisteredModels};
pub use registry::*; // re-exporta la macro register_models!
