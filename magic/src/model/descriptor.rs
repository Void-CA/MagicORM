use serde::Serialize;
use crate::model::meta::{ColumnMeta, ForeignKeyMeta};

// ---------------------------------------------------------------------------
// ModelDescriptor — snapshot serializable de un modelo para el registro
// ---------------------------------------------------------------------------

#[derive(Serialize, Debug)]
pub struct ModelDescriptor {
    pub table: &'static str,
    pub columns: &'static [ColumnMeta],
    pub foreign_keys: &'static [ForeignKeyMeta],
}

// ---------------------------------------------------------------------------
// RegisteredModels — trait implementado por `register_models!(...)`.
// Agrupa todos los modelos de la aplicación en un solo punto.
// ---------------------------------------------------------------------------

pub trait RegisteredModels {
    fn models() -> Vec<ModelDescriptor>;
}
