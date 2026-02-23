extern crate proc_macro;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod attrs;
mod model;
mod expand;
mod operations;
mod generators;


use attrs::{parse_model_fks, parse_magic_attributes};
use model::analyze_model;
use expand::expand_magic_model;

macro_rules! unwrap_or_ts {
    ($expr:expr) => {
        match $expr {
            Ok(v) => v,
            Err(err) => return err.to_compile_error().into(),
        }
    };
}

#[proc_macro_derive(MagicModel, attributes(magic, FK))]
pub fn derive_magic_model(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let config = unwrap_or_ts!(parse_magic_attributes(&input));

    let model = unwrap_or_ts!(analyze_model(&input));

    let fk_fields= unwrap_or_ts!(parse_model_fks(&model));

    expand_magic_model(&input, config, model, &fk_fields).into()
}

