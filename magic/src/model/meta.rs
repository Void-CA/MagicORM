use serde::Serialize;

// ---------------------------------------------------------------------------
// ColumnMeta — descriptor estático de una columna
// ---------------------------------------------------------------------------

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct ColumnMeta {
    pub name: &'static str,
    pub sql_type: &'static str,
    pub nullable: bool,
    pub primary_key: bool,
    pub auto_increment: bool,
}

// ---------------------------------------------------------------------------
// ForeignKeyMeta — descriptor estático de una clave foránea
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct ForeignKeyMeta {
    pub field: &'static str,
    pub related_column: &'static str,
    pub related_table: &'static str,
}

// ---------------------------------------------------------------------------
// ModelMeta — trait base de metadatos de tiempo de compilación
// Implementado automáticamente por `#[derive(MagicModel)]`.
// ---------------------------------------------------------------------------

pub trait ModelMeta {
    const TABLE: &'static str;
    fn foreign_keys() -> &'static [ForeignKeyMeta];
    fn columns() -> &'static [ColumnMeta];
}
