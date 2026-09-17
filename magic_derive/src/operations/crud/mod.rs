pub(crate) mod delete;
pub(crate) mod get;
pub(crate) mod insert;
pub(crate) mod put;
pub(crate) mod upsert;

pub use delete::{generate_delete, generate_delete_by_id};
pub use get::{generate_get, generate_get_by_id};
pub use insert::{generate_insert, generate_insert_many, generate_newstruct_insert};
pub use put::{generate_newstruct_put, generate_put};
pub use upsert::{generate_newstruct_upsert, generate_upsert, generate_upsert_many};
