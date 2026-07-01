use std::time::Instant;

use crate::model::{ModelMeta, ModelDescriptor, RegisteredModels};
use sqlx::{Executor, IntoArguments};
use std::collections::HashSet;
use tracing::debug;

/// Genera SQL de creación de tabla
pub fn create_table_sql<T: ModelMeta>() -> String {
    let mut sql = format!("CREATE TABLE IF NOT EXISTS {} (\n", T::TABLE);
    let mut column_defs = Vec::new();
    let mut foreign_keys = Vec::new();

    for col in T::columns() {
        let mut def = format!("    {} {}", col.name, col.sql_type);
        if col.primary_key {
            def.push_str(" PRIMARY KEY");
        }
        if !col.nullable && !col.primary_key {
            def.push_str(" NOT NULL");
        }
        column_defs.push(def);

        for fk in T::foreign_keys() {
            if fk.field == col.name {
                foreign_keys.push(format!(
                    "    FOREIGN KEY({}) REFERENCES {}({}) ON DELETE CASCADE",
                    fk.field, fk.related_table, fk.related_column
                ));
            }
        }
    }

    column_defs.extend(foreign_keys);
    sql.push_str(&column_defs.join(",\n"));
    sql.push_str("\n);");
    sql
}


/// Genera SQL de creación de tabla a partir de un descriptor
pub fn create_table_sql_from_descriptor(desc: &ModelDescriptor) -> String {
    let mut sql = format!("CREATE TABLE IF NOT EXISTS {} (\n", desc.table);
    let mut column_defs = Vec::new();
    let mut foreign_keys = Vec::new();

    for col in desc.columns {
        let mut def = format!("    {} {}", col.name, col.sql_type);
        if col.primary_key {
            def.push_str(" PRIMARY KEY");
        }
        if !col.nullable && !col.primary_key {
            def.push_str(" NOT NULL");
        }
        column_defs.push(def);
    }

    for fk in desc.foreign_keys {
        foreign_keys.push(format!(
            "    FOREIGN KEY({}) REFERENCES {}({}) ON DELETE CASCADE",
            fk.field, fk.related_table, fk.related_column
        ));
    }

    column_defs.extend(foreign_keys);
    sql.push_str(&column_defs.join(",\n"));
    sql.push_str("\n);");
    sql
}

pub async fn create_all<'e, E, R>(executor: E) -> anyhow::Result<()>
where
    E: Executor<'e> + Copy,
    R: RegisteredModels,
    for<'q> <E::Database as sqlx::Database>::Arguments<'q>: sqlx::IntoArguments<'q, E::Database>,
{
    let start = Instant::now();
    let mut models = R::models();
    debug!(model_count = models.len(), "create_all started");
    let mut created = HashSet::new();

    while !models.is_empty() {
        let mut ready_indices = Vec::new();

        for (idx, model) in models.iter().enumerate() {
            let deps: Vec<&str> = model
                .foreign_keys
                .iter()
                .map(|fk| fk.related_table)
                .collect();

            if deps.iter().all(|d| created.contains(d)) {
                ready_indices.push(idx);
            }
        }

        if ready_indices.is_empty() {
            debug!("create_all: cycle detected, remaining models: {:?}", models.iter().map(|m| m.table).collect::<Vec<_>>());
            anyhow::bail!("Schema cycle detected");
        }

        for &idx in &ready_indices {
            let model = &models[idx];
            let sql = create_table_sql_from_descriptor(model);
            debug!(table = model.table, sql = %sql, "create_all: creating table");

            sqlx::query(&sql).execute(executor).await?;

            debug!(table = model.table, "create_all: table created");
            created.insert(model.table);
        }

        for &idx in &ready_indices {
            let model = &models[idx];
            let sql = create_table_sql_from_descriptor(model);

            executor.execute(sql.as_str()).await?;

            created.insert(model.table);
        }

        for &idx in ready_indices.iter().rev() {
            models.remove(idx);
        }
    }

    let elapsed = start.elapsed();
    debug!(elapsed_us = elapsed.as_micros() as u64, "create_all completed");
    Ok(())
}
