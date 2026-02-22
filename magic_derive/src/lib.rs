extern crate proc_macro;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod attrs;
mod model;
mod expand;
mod crud;

use attrs::parse_magic_attributes;
use model::analyze_model;
use expand::expand_magic_model;

#[proc_macro_derive(MagicModel, attributes(magic))]
pub fn derive_magic_model(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let config = match parse_magic_attributes(&input) {
        Ok(cfg) => cfg,
        Err(err) => return err.to_compile_error().into(),
    };

    let model = match analyze_model(&input) {
        Ok(model) => model,
        Err(err) => return err.to_compile_error().into(),
    };

    expand_magic_model(&input, config, model).into()
}