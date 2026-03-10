use proc_macro::TokenStream;
use proc_macro2::{Literal, Span};
use quote::quote;
use syn::{Expr, Ident, Item};

pub fn gen_tuple_into_component_vec_watchables_impl(_data: TokenStream) -> TokenStream {
    let mut implementations = Vec::<Item>::new();
    for size in 0..=10 {
        if size == 1 {
            continue;
        }
        let mut tuple = Vec::<Ident>::new();
        let mut fields = Vec::<Expr>::new();
        for i in 0..size {
            let lit = Literal::i32_unsuffixed(i);
            let gen = Ident::new(&format!("G{i}"), Span::call_site());
            fields.push(syn::parse_quote!(self.#lit));
            tuple.push(gen);
        }

        implementations.push(syn::parse_quote!(
            impl<#(#tuple: Into<Component>),*> IntoComponentVec for (#(#tuple),*) {
                fn into_vec(self) -> Vec<Component> {
                    vec![#(#fields.into()),*]
                }
            }
        ));
        implementations.push(syn::parse_quote!(
            into_component_vec_watchable!(<#(#tuple),*>, (#(#tuple),*));
        ));
    }

    TokenStream::from(quote! {
        #(#implementations)*
    })
}
