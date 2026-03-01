use crate::meta::ModelMeta;

use crate::registry::ModelDescriptor;

pub fn dependencies(model: &ModelDescriptor) -> Vec<&'static str> {
    model
        .foreign_keys
        .iter()
        .map(|fk| fk.related_table)
        .collect()
}

pub fn infer_fk(column_name: &str) -> Option<&str> {
    if column_name.ends_with("_id") && column_name != "id" {
        Some(column_name.trim_end_matches("_id"))
    } else {
        None
    }
}