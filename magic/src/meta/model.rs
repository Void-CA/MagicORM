use crate::meta::{ForeignKeyMeta, column::ColumnMeta};

pub trait ModelMeta {
    const TABLE: &'static str;
    fn foreign_keys() -> &'static [ForeignKeyMeta];
    fn columns() -> &'static [ColumnMeta];
}