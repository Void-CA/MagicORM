#[derive(Debug)]
pub enum SchemaError<E> {
    Executor(E),
    CycleDetected,
}


impl<E: std::error::Error + Send + Sync + 'static> From<SchemaError<E>> for anyhow::Error {
    fn from(e: SchemaError<E>) -> Self {
        match e {
            SchemaError::Executor(inner) => anyhow::anyhow!(inner),
            SchemaError::CycleDetected => anyhow::anyhow!("Schema cycle detected"),
        }
    }
}