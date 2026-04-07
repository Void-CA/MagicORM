use crate::model::Model;
use crate::query::QueryBuilder;
use std::collections::HashMap;
use std::marker::PhantomData;

pub struct EagerQueryBuilder<'a, P, C> {
    pub base: QueryBuilder<'a, P>,
    pub _marker: PhantomData<C>,
}

pub struct WithMany<P: Model<Id = i64>, C> {
    pub parents: Vec<P>,
    pub children: HashMap<P::Id, Vec<C>>,
}