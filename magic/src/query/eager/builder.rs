use crate::model::Model;
use crate::prelude::HasFK;
use crate::query::QueryBuilder;
use std::collections::HashMap;
use std::marker::PhantomData;

pub struct EagerQueryBuilder<'a, P, C> {
    pub base: QueryBuilder<'a, P>,
    pub _marker: PhantomData<C>,
}

#[derive(Debug)]
pub struct WithMany<P: Model<Id = i64>, C> {
    pub parents: Vec<P>,
    pub children: HashMap<P::Id, Vec<C>>,
}

impl<P, C> WithMany<P, C>
where
    P: Model<Id = i64>,
    C: Model + HasFK<P>,
{
    pub fn children_of(&self, parent: &P) -> &[C] {
        self.children
            .get(&parent.id())
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    pub fn iter(&self) -> impl Iterator<Item = (&P, &[C])> {
    self.parents.iter().map(move |p| {
        let children = self
            .children
            .get(&p.id())
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        (p, children)
    })
}
}