use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ForeignKeyMeta {
    pub field: &'static str,
    pub related_column: &'static str,
    pub related_table: &'static str,
}