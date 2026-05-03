use crate::model::Model;
use crate::prelude::HasFK;
use crate::query::QueryBuilder;
use std::collections::HashMap;
use std::marker::PhantomData;

/// EagerQueryBuilder permite encadenar filtros y luego cargar relaciones con `fetch_all`.
/// Utiliza composición interna con `inner: QueryBuilder` y delegación manual.
pub struct EagerQueryBuilder<'a, P: Model<Id = i64>, C> {
    pub inner: QueryBuilder<'a, P>,
    pub _marker: PhantomData<C>,
}

// Delegación manual de métodos necesarios
impl<'a, P: Model<Id = i64>, C> EagerQueryBuilder<'a, P, C> {
    pub fn filter(mut self, col: &str, op: &str, value: impl std::string::ToString) -> Self {
        self.inner = self.inner.filter(col, op, value);
        self
    }

    pub fn order_by(mut self, col: &str, asc: bool) -> Self {
        self.inner = self.inner.order_by(col, asc);
        self
    }

    pub fn limit(mut self, lim: u32) -> Self {
        self.inner = self.inner.limit(lim);
        self
    }

    pub fn offset(mut self, off: u32) -> Self {
        self.inner = self.inner.offset(off);
        self
    }
}

#[derive(Debug)]
pub struct WithMany<P: Model<Id = i64>, C> {
    pub parents: Vec<P>,
    pub children: HashMap<i64, Vec<C>>,
}

impl<P, C> WithMany<P, C>
where
    P: Model<Id = i64>,
    C: Model,
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
