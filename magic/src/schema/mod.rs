pub mod create;
pub mod utils;
pub mod migration;

#[cfg(feature = "sqlite")]
pub mod introspect;

// ModelDescriptor y RegisteredModels viven en model::descriptor;
// los re-exportamos aquí para mantener compatibilidad de paths existentes.
pub use crate::model::{ModelDescriptor, RegisteredModels};
pub use create::*;