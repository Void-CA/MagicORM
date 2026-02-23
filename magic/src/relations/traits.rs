pub trait HasRelations {
    type HasMany: RelationList;
}

pub trait RelationList {
    fn all() -> Vec<&'static str>;
}