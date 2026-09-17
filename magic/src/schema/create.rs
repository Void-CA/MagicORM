use std::time::Instant;

use crate::model::{IndexMeta, ModelDescriptor, ModelMeta, RegisteredModels};
use sqlx::Executor;
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

/// Genera SQL de creación de índice
pub fn create_index_sql(table: &str, index: &IndexMeta) -> String {
    let unique = if index.unique { "UNIQUE " } else { "" };
    let cols = index.columns.join(", ");
    format!(
        "CREATE {}INDEX IF NOT EXISTS {} ON {} ({})",
        unique, index.name, table, cols
    )
}

/// Genera SQL de creación de índices para un modelo
pub fn create_indexes_sql<T: ModelMeta>() -> Vec<String> {
    T::indexes()
        .iter()
        .map(|idx| create_index_sql(T::TABLE, idx))
        .collect()
}

/// Genera SQL de creación de tabla a partir de un descriptor
pub fn create_table_sql_from_descriptor(desc: &ModelDescriptor) -> String {
    let mut sql = format!("CREATE TABLE IF NOT EXISTS {} (\n", desc.table);
    let mut column_defs = Vec::new();
    let mut foreign_keys = Vec::new();

    for col in &desc.columns {
        let mut def = format!("    {} {}", col.name, col.sql_type);
        if col.primary_key {
            def.push_str(" PRIMARY KEY");
        }
        if !col.nullable && !col.primary_key {
            def.push_str(" NOT NULL");
        }
        column_defs.push(def);
    }

    for fk in &desc.foreign_keys {
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
    let mut created: HashSet<String> = HashSet::new();

    // Create tables first
    while !models.is_empty() {
        let mut ready_indices = Vec::new();

        for (idx, model) in models.iter().enumerate() {
            let deps: Vec<String> = model
                .foreign_keys
                .iter()
                .map(|fk| fk.related_table.clone())
                .collect();

            if deps.iter().all(|d| created.contains(d)) {
                ready_indices.push(idx);
            }
        }

        if ready_indices.is_empty() {
            debug!(
                "create_all: cycle detected, remaining models: {:?}",
                models.iter().map(|m| &m.table).collect::<Vec<_>>()
            );
            anyhow::bail!("Schema cycle detected");
        }

        for &idx in ready_indices.iter().rev() {
            let model = models.remove(idx);
            let sql = create_table_sql_from_descriptor(&model);
            debug!(table = %model.table, sql = %sql, "create_all: creating table");

            sqlx::query(&sql).execute(executor).await?;

            debug!(table = %model.table, "create_all: table created");
            created.insert(model.table);
        }
    }

    // Create indexes after all tables are created
    let schema = R::schema();
    for model in &schema.models {
        for index in &model.indexes {
            let sql = create_index_sql(&model.table, index);
            debug!(table = %model.table, index = %index.name, sql = %sql, "create_all: creating index");
            sqlx::query(&sql).execute(executor).await?;
            debug!(table = %model.table, index = %index.name, "create_all: index created");
        }
    }

    let elapsed = start.elapsed();
    debug!(
        elapsed_us = elapsed.as_micros() as u64,
        "create_all completed"
    );
    Ok(())
}
