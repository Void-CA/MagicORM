pub(crate) mod attrs;
pub(crate) mod parser;

pub use attrs::{FKConfig, MagicConfig, parse_magic_attributes, parse_model_fks};
pub use parser::{FieldInfo, ModelInfo, analyze_model};
