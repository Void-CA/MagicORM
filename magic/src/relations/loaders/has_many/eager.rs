use std::collections::HashMap;

use crate::{
    model::{Model, ModelMeta},
    relations::traits::HasFK,
};

pub async fn load_has_many_batch<'e, P, C, E>(
    parents: &[P],
    executor: E,
) -> anyhow::Result<HashMap<P::Id, Vec<C>>>
where
    P: Model<Id = i64>,
    C: Model
        + ModelMeta
        + HasFK<P>
        + for<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow>
        + Send
        + Unpin,
    E: sqlx::Executor<'e, Database = sqlx::Sqlite>,

{
    if parents.is_empty() {
        return Ok(HashMap::new());
    }

    // recolectar ids
    let mut ids: Vec<i64> = parents.iter().map(|p| *p.id()).collect();
    ids.sort();
    ids.dedup();

    // construir query IN (...)
    let fk_column = C::fk_for_parent();

    let placeholders = vec!["?"; ids.len()].join(", ");
    let sql = format!(
        "SELECT * FROM {} WHERE {} IN ({})",
        C::TABLE,
        fk_column,
        placeholders
    );

    let mut query = sqlx::query_as::<_, C>(&sql);

    for id in &ids {
        query = query.bind(id);
    }

    // ejecutar
    let rows = query
        .fetch_all(executor)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;

    // agrupar
    let mut map: HashMap<P::Id, Vec<C>> = HashMap::with_capacity(parents.len());

    for row in rows {
        let key = row.fk_value(); 
        map.entry(key).or_default().push(row);
    }

    Ok(map)
}