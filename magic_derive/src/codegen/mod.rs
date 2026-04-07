pub(crate) mod expand;
pub(crate) mod crud_methods;
pub(crate) mod registry_method;
pub(crate) mod impl_from_row;
pub(crate) mod impl_model;
pub(crate) mod impl_model_meta;
pub(crate) mod impl_has_fk;
pub(crate) mod utils;
pub(crate) mod impl_belongs_to;

pub use expand::expand_magic_model;
