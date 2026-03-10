use proc_macro::TokenStream;

mod builder;
mod component_vec;
mod inheritable;
mod wasm;

use crate::{
    builder::{builder_into_comp_impl, watchable_setters_impl},
    component_vec::gen_tuple_into_component_vec_watchables_impl,
    inheritable::derive_inheritable_impl,
    wasm::wasm_getters_impl,
};

#[proc_macro_attribute]
pub fn wasm_getters(attr: TokenStream, item: TokenStream) -> TokenStream {
    wasm_getters_impl(attr, item)
}

#[proc_macro_attribute]
pub fn builder_into_comp(attr: TokenStream, item: TokenStream) -> TokenStream {
    builder_into_comp_impl(attr, item)
}

#[proc_macro_attribute]
pub fn watchable_setters(attr: TokenStream, item: TokenStream) -> TokenStream {
    watchable_setters_impl(attr, item)
}

#[proc_macro]
pub fn gen_tuple_into_component_vec_watchables(data: TokenStream) -> TokenStream {
    gen_tuple_into_component_vec_watchables_impl(data)
}

#[proc_macro_derive(Inheritable)]
pub fn derive_inheritable(input: TokenStream) -> TokenStream {
    derive_inheritable_impl(input)
}
