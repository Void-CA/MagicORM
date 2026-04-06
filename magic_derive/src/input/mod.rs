pub(crate) mod parser;
pub(crate) mod attrs;

pub use parser::{analyze_model, ModelInfo, FieldInfo};
pub use attrs::{parse_model_fks, parse_magic_attributes, MagicConfig, FKConfig};
