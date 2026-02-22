pub mod magic;
pub mod foreign_key;

pub use magic::{MagicConfig, parse_magic_attributes};
pub use foreign_key::{FKConfig, parse_model_fks};
