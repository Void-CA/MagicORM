#[derive(Debug)]
pub enum SchemaError<E> {
    Executor(E),
    CycleDetected,
}