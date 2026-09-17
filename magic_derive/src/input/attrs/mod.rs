pub mod foreign_key;
pub mod magic;

pub use foreign_key::{FKConfig, parse_model_fks};
pub use magic::{IndexConfig, MagicConfig, parse_magic_attributes};
