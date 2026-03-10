use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

pub fn derive_inheritable_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;

    let body = match input.data {
        Data::Struct(data) => inherit_struct(&data.fields),
        Data::Enum(data) => inherit_enum(&name, &data.variants),
        _ => panic!("Can only derive inheritable for Structs and Enums"),
    };

    let expanded = quote! {
        impl crate::inputs::inherited_input::Inheritable for #name {
            fn inherit(&self, self_name: impl crate::util::watchables::watchable_utils::IntoWatchable<crate::inputs::inherited_input::InheritLabel> + Clone + 'static) -> Self {
                #body
            }
        }
    };

    expanded.into()
}
fn inherit_struct(fields: &syn::Fields) -> proc_macro2::TokenStream {
    match fields {
        Fields::Named(fields) => {
            let clones = fields.named.iter().map(|f| {
                let name = &f.ident;
                quote! {
                    #name: crate::inputs::inherited_input::Inheritable::inherit(&self.#name, self_name.clone())
                }
            });
            quote! { Self { #(#clones),*} }
        }
        Fields::Unnamed(fields) => {
            let clones = fields.unnamed.iter().enumerate().map(|(i, _)| {
                let idx = syn::Index::from(i);
                quote! {
                    crate::inputs::inherited_input::Inheritable::inherit(&self.#idx, self_name.clone())
                }
            });
            quote! { Self( #(#clones),* ) }
        }
        Fields::Unit => quote! { Self },
    }
}
fn inherit_enum(
    name: &syn::Ident,
    variants: &syn::punctuated::Punctuated<syn::Variant, syn::token::Comma>,
) -> proc_macro2::TokenStream {
    let arms = variants.iter().map(|variant| {
        let vname = &variant.ident;

        match &variant.fields {
            Fields::Named(fields) => {
                let names: Vec<_> = fields
                    .named
                    .iter()
                    .map(|f| f.ident.as_ref().unwrap())
                    .collect();
                let clones = names.iter().map(|n| {
                    quote! { #n: crate::inputs::inherited_input::Inheritable::inherit(&#n, self_name.clone()) }
                });
                quote! {
                    #name::#vname { #( ref #names ),* } => {
                        #name::#vname {
                            #(#clones),*
                        }
                    }
                }
            }

            Fields::Unnamed(fields) => {
                let bindings: Vec<_> = (0..fields.unnamed.len())
                    .map(|i| syn::Ident::new(&format!("f{i}"), proc_macro2::Span::call_site()))
                    .collect();

                let clones = bindings.iter().map(|b| {
                    quote! { crate::inputs::inherited_input::Inheritable::inherit(&#b, self_name.clone()) }
                });

                quote! {
                    #name::#vname( #( ref #bindings ),* ) => {
                        #name::#vname(
                            #(#clones),*
                        )
                    }
                }
            }

            Fields::Unit => { quote! { #name::#vname => #name::#vname }}
        }
    });

    quote! {
        match self {
            #(#arms),*
        }
    }
}
