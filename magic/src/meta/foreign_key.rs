pub struct ForeignKeyMeta {
    pub field: &'static str,
    pub related_column: &'static str,
    pub related_table: fn() -> &'static str,
}