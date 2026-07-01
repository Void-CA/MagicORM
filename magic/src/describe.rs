use crate::model::ModelDescriptor;

// ---------------------------------------------------------------------------
// Describe — trait para exponer metadatos del modelo en runtime.
// Implementado automáticamente por `#[derive(MagicModel)]`.
// ---------------------------------------------------------------------------
pub trait Describe {
    fn descriptor() -> ModelDescriptor;
}

// ---------------------------------------------------------------------------
// Helper para serializar todos los descriptores a JSON.
// ---------------------------------------------------------------------------
pub fn descriptors_to_json(models: &[ModelDescriptor]) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(models)
}

pub fn descriptor_to_json(desc: &ModelDescriptor) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(desc)
}
