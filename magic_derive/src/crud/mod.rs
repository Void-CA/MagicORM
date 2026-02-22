pub mod insert;
pub mod update;

pub use insert::{generate_insert, generate_newstruct_insert};
pub use update::{generate_update, generate_newstruct_update};