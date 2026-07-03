use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// ColumnMeta — descriptor de una columna
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ColumnMeta {
    pub name: String,
    pub sql_type: String,
    pub nullable: bool,
    pub primary_key: bool,
    pub auto_increment: bool,
}

// ---------------------------------------------------------------------------
// ForeignKeyMeta — descriptor de una clave foránea
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ForeignKeyMeta {
    pub field: String,
    pub related_column: String,
    pub related_table: String,
}

// ---------------------------------------------------------------------------
// ModelMeta — trait base de metadatos de tiempo de compilación
// Implementado automáticamente por `#[derive(MagicModel)]`.
// Retorna Vec para que ColumnMeta sea completamente owned.
// ---------------------------------------------------------------------------

pub trait ModelMeta {
    const TABLE: &'static str;
    fn foreign_keys() -> Vec<ForeignKeyMeta>;
    fn columns() -> Vec<ColumnMeta>;
}
