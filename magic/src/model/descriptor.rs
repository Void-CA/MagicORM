use serde::Serialize;
use crate::model::meta::{ColumnMeta, ForeignKeyMeta};

// ---------------------------------------------------------------------------
// ModelDescriptor — snapshot serializable de un modelo.
// Usa tipos owned (String, Vec) para ser construible en runtime.
// ---------------------------------------------------------------------------

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct ModelDescriptor {
    pub table: String,
    pub columns: Vec<ColumnMeta>,
    pub foreign_keys: Vec<ForeignKeyMeta>,
}

// ---------------------------------------------------------------------------
// SchemaDescriptor — describe todos los modelos del proyecto.
// ---------------------------------------------------------------------------

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct SchemaDescriptor {
    pub models: Vec<ModelDescriptor>,
}

impl SchemaDescriptor {
    pub fn new(models: Vec<ModelDescriptor>) -> Self {
        Self { models }
    }
}

// ---------------------------------------------------------------------------
// RegisteredModels — trait implementado por `register_models!(...)`.
// ---------------------------------------------------------------------------

pub trait RegisteredModels {
    fn models() -> Vec<ModelDescriptor>;
    fn schema() -> SchemaDescriptor {
        SchemaDescriptor::new(Self::models())
    }
}
