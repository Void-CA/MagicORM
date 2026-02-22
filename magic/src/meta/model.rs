use crate::meta::ForeignKeyMeta;

pub trait ModelMeta {
    fn table() -> &'static str;
    fn foreign_keys() -> &'static [ForeignKeyMeta];
}