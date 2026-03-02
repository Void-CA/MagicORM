
use crate::schema::ModelDescriptor;

pub fn dependencies(model: &ModelDescriptor) -> Vec<&'static str> {
    model
        .foreign_keys
        .iter()
        .map(|fk| fk.related_table)
        .collect()
}
