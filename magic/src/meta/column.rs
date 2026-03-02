use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct ColumnMeta {
    pub name: &'static str,
    pub sql_type: &'static str,
    pub nullable: bool,
    pub primary_key: bool,
}