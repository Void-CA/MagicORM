pub struct ForeignKeyMeta {
    pub field: &'static str,
    pub related_table: &'static str,
    pub related_column: &'static str,
}