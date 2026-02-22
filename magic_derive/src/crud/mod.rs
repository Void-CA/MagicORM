pub mod insert;
pub mod update;
pub mod select;
pub mod delete;

pub use insert::{generate_insert, generate_newstruct_insert};
pub use update::{generate_update, generate_newstruct_update};
pub use select::{generate_select, generate_select_by_id};
pub use delete::{generate_delete, generate_delete_by_id};