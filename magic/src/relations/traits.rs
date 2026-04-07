use crate::model::Model;

pub trait HasRelations {
    type HasMany: RelationList;
}

pub trait RelationList {
    fn all() -> Vec<&'static str>;
}

pub trait HasFK<P: Model> {
    fn fk_for_parent() -> &'static str;

    fn fk_value(&self) -> P::Id;
}