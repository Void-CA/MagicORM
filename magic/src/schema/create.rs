// schema.rs
use crate::executor::Executor;
use crate::meta::ModelMeta;
use crate::schema::RegisteredModels;
use std::collections::HashSet;

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
                    "    FOREIGN KEY({}) REFERENCES {}({})",
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

/// Crea todas las tablas usando los modelos registrados
pub async fn create_all<E, R>(executor: &E) -> anyhow::Result<()>
where
    E: Executor,
    R: RegisteredModels,
{
    let mut models = R::models();
    let mut created = HashSet::new();

    while !models.is_empty() {
        let mut ready_indices = Vec::new();

        for (idx, model) in models.iter().enumerate() {
            let deps: Vec<&str> = model
                .columns
                .iter()
                .filter_map(|c| if c.name.ends_with("_id") && c.name != "id" { Some(c.name.trim_end_matches("_id")) } else { None })
                .collect();

            if deps.iter().all(|d| created.contains(d)) {
                ready_indices.push(idx);
            }
        }

        if ready_indices.is_empty() {
            anyhow::bail!("Schema cycle detected");
        }

        for &idx in &ready_indices {
            let model = &models[idx];
            let sql = format!(
                "CREATE TABLE IF NOT EXISTS {} (...);", // o genera con create_table_sql si quieres
                model.table
            );
            executor.execute(&sql).await?;
            created.insert(model.table);
        }

        for &idx in ready_indices.iter().rev() {
            models.remove(idx);
        }
    }

    Ok(())
}