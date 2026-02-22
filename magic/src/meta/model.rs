use crate::meta::ForeignKeyMeta;

pub trait ModelMeta {
    const TABLE: &'static str;
    fn foreign_keys() -> &'static [ForeignKeyMeta];
}