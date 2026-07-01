use std::collections::HashMap;
use std::time::Instant;

use crate::model::Model;
use crate::relations::traits::HasFK;
use tracing::debug;

/// Eager loading: carga todos los hijos de todos los padres en una sola query.
pub async fn load_has_many_batch<'e, P, C>(
    parents: &[P],
    executor: impl sqlx::Executor<'e, Database = P::DB>,
) -> anyhow::Result<HashMap<P::Id, Vec<C>>>
where
    P: Model,
    C: Model<DB = P::DB> + HasFK<P>,
    P::Id: Clone + Eq + std::hash::Hash,
    for<'q> <P::DB as sqlx::Database>::Arguments<'q>: sqlx::IntoArguments<'q, P::DB>,
{
    let start = Instant::now();

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
    debug!(table = C::TABLE, fk_column, parent_count = parents.len(), distinct_ids = ids.len(), "load_has_many_batch");

    let mut query = sqlx::query_as::<_, C>(&sql);
    for id in &ids {
        query = query.bind(id.clone());
    }

    let result = query.fetch_all(executor).await;
    let elapsed = start.elapsed();

    match result {
        Ok(rows) => {
            let mut map: HashMap<P::Id, Vec<C>> = HashMap::with_capacity(parents.len());
            for row in rows {
                let key = row.fk_value();
                map.entry(key).or_default().push(row);
            }
            debug!(groups = map.len(), elapsed_us = elapsed.as_micros() as u64, "load_has_many_batch done");
            Ok(map)
        }
        Err(e) => {
            debug!(error = %e, elapsed_us = elapsed.as_micros() as u64, "load_has_many_batch failed");
            Err(anyhow::anyhow!(e))
        }
    }
}
