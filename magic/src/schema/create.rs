use std::collections::HashSet;

use crate::executor::{Executor, SchemaError};
use crate::registry::{ModelDescriptor, RegisteredModels};
use crate::schema::utils::{infer_fk, dependencies};

pub fn create_table_sql(model: &ModelDescriptor) -> String {
    let mut sql = format!(
        "CREATE TABLE IF NOT EXISTS {} (\n",
        model.table
    );

    let mut parts = Vec::new();

    for col in model.columns {
        let mut def = format!("    {} {}", col.name, col.sql_type);

        if col.primary_key {
            def.push_str(" PRIMARY KEY");
        }

        if !col.nullable && !col.primary_key {
            def.push_str(" NOT NULL");
        }

        parts.push(def);
    }

    for fk in model.foreign_keys {
        parts.push(format!(
            "    FOREIGN KEY ({}) REFERENCES {}({}) ON DELETE CASCADE",
            fk.field,
            fk.related_table,
            fk.related_column
        ));
    }

    sql.push_str(&parts.join(",\n"));
    sql.push_str("\n);");

    sql
}


pub async fn create_all<E, R>(
    executor: &E,
) -> Result<(), SchemaError<E::Error>>
where
    E: Executor,
    R: RegisteredModels,
{
    let mut models = R::models();
    let mut created = HashSet::new();

    while !models.is_empty() {
        let mut ready_indices = Vec::new();

        for (idx, model) in models.iter().enumerate() {
            let deps = dependencies(model);

            if deps.iter().all(|d| created.contains(d)) {
                ready_indices.push(idx);
            }
        }

        if ready_indices.is_empty() {
            return Err(SchemaError::CycleDetected);
        }

        for &idx in &ready_indices {
            let model = &models[idx];

            let sql = create_table_sql(model);
            executor.execute(&sql).await.map_err(SchemaError::Executor)?;

            created.insert(model.table);
        }

        for &idx in ready_indices.iter().rev() {
            models.remove(idx);
        }
    }

    Ok(())
}
