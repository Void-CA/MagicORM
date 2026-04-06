pub(crate) mod insert;
pub(crate) mod put;
pub(crate) mod get;
pub(crate) mod delete;

pub use insert::{generate_insert, generate_newstruct_insert};
pub use put::{generate_put, generate_newstruct_put};
pub use get::{generate_get, generate_get_by_id};
pub use delete::{generate_delete, generate_delete_by_id};