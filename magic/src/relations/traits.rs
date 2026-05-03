use crate::model::Model;

pub trait HasRelations {
    type HasMany: RelationList;
}

pub trait RelationList {
    fn all() -> Vec<&'static str>;
}

pub trait HasFK<P>
where
    P: Model,
    P::Id: Clone + Eq + std::hash::Hash,
    P::DB: sqlx::Database,
{
    fn fk_for_parent() -> &'static str;
    fn fk_value(&self) -> P::Id;
}