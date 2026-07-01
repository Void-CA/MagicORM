use std::collections::HashMap;
use sqlx::{Database, IntoArguments};
use crate::model::Model;
use crate::relations::traits::HasFK;

/// Eager loading: carga todos los hijos de todos los padres en una sola query.
pub async fn load_has_many_batch<'e, P, C>(
    parents: &[P],
    executor: impl sqlx::Executor<'e, Database = P::DB>,
) -> anyhow::Result<HashMap<P::Id, Vec<C>>>
where
    P: Model,
    C: Model<DB = P::DB> + HasFK<P>,
    P::Id: Clone + Eq + std::hash::Hash,
    for<'q> <P::DB as Database>::Arguments<'q>: IntoArguments<'q, P::DB>,
{
    if parents.is_empty() {
        return Ok(HashMap::new());
    }

    let mut ids: Vec<P::Id> = parents.iter().map(|p| p.id().clone()).collect();
    ids.sort_by(|a, b| a.to_string().cmp(&b.to_string()));
    ids.dedup();

    let fk_column = C::fk_for_parent();
    let placeholders = vec!["?"; ids.len()].join(", ");
    let sql = format!(
        "SELECT * FROM {} WHERE {} IN ({})",
        C::TABLE, fk_column, placeholders
    );

    let mut query = sqlx::query_as::<_, C>(&sql);
    for id in &ids {
        query = query.bind(id.clone());
    }

    let rows = query
        .fetch_all(executor)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;

    let mut map: HashMap<P::Id, Vec<C>> = HashMap::with_capacity(parents.len());
    for row in rows {
        let key = row.fk_value();
        map.entry(key).or_default().push(row);
    }

    Ok(map)
}
