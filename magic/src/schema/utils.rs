use crate::schema::ModelDescriptor;

pub fn dependencies(model: &ModelDescriptor) -> Vec<String> {
    model
        .foreign_keys
        .iter()
        .map(|fk| fk.related_table.clone())
        .collect()
}
