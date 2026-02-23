pub trait HasRelations {
    type HasMany: RelationList;
}

pub trait RelationList {
    fn all() -> Vec<&'static str>;
}

pub trait HasFK<P> {
    fn fk_for_parent() -> &'static str;
}