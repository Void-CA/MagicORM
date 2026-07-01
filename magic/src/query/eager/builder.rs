use crate::model::Model;
use crate::query::QueryBuilder;
use std::collections::HashMap;
use std::marker::PhantomData;

use sqlx::Database;

/// EagerQueryBuilder permite encadenar filtros y luego cargar relaciones con `fetch_all`.
pub struct EagerQueryBuilder<'a, DB: Database, P: Model<DB = DB>, C> {
    pub inner: QueryBuilder<'a, DB, P>,
    pub _marker: PhantomData<C>,
}

// ---------------------------------------------------------------------------
// ConWithMany — resultado de eager loading con padres e hijos agrupados.
// ---------------------------------------------------------------------------
#[derive(Debug)]
pub struct WithMany<P: Model, C> {
    pub parents: Vec<P>,
    pub children: HashMap<P::Id, Vec<C>>,
}

impl<P, C> WithMany<P, C>
where
    P: Model,
    P::Id: Clone + Eq + std::hash::Hash,
    C: Model,
{
    pub fn children_of(&self, parent: &P) -> &[C] {
        self.children
            .get(parent.id())
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    pub fn iter(&self) -> impl Iterator<Item = (&P, &[C])> {
        self.parents.iter().map(move |p| {
            let children = self
                .children
                .get(p.id())
                .map(Vec::as_slice)
                .unwrap_or(&[]);
            (p, children)
        })
    }
}
