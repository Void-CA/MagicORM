pub(crate) mod attrs;
pub(crate) mod parser;

pub use attrs::FKConfig;
pub use parser::{FieldInfo, ModelInfo, analyze_model};
