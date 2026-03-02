use crate::meta::ColumnMeta;
use serde::Serialize;

/// Descriptor de modelo para registro
#[derive(Serialize, Debug)]
pub struct ModelDescriptor {
    pub table: &'static str,
    pub columns: &'static [ColumnMeta],
    pub foreign_keys: &'static [crate::meta::ForeignKeyMeta],
}

/// Registro explícito de modelos
pub trait RegisteredModels {
    fn models() -> Vec<ModelDescriptor>;
}
