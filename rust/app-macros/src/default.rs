use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Field, Fields, Type, TypePath};

pub fn derive_default_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let body = match input.data {
        Data::Struct(data) => default_struct(&data.fields),
        Data::Enum(data) => default_enum(&name, &data.variants),
        _ => panic!("Can only derive Default for structs and enums"),
    };

    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics core::default::Default for #name #ty_generics #where_clause {
            fn default() -> Self {
                #body
            }
        }
    };

    expanded.into()
}
fn default_struct(fields: &syn::Fields) -> proc_macro2::TokenStream {
    match fields {
        Fields::Named(fields) => {
            let defaults = fields.named.iter().map(|f| {
                let init = get_init_code(f);
                let name = &f.ident;
                quote! { #name: #init }
            });
            quote! { Self { #(#defaults),*} }
        }
        Fields::Unnamed(fields) => {
            let defaults = fields.unnamed.iter().map(get_init_code);
            quote! { Self( #(#defaults),* ) }
        }
        Fields::Unit => quote! { Self },
    }
}
fn default_enum(
    name: &syn::Ident,
    variants: &syn::punctuated::Punctuated<syn::Variant, syn::token::Comma>,
) -> proc_macro2::TokenStream {
    let arms = variants.iter().map(|variant| {
        let vname = &variant.ident;

        match &variant.fields {
            Fields::Named(fields) => {
                let defaults = fields.named.iter().map(|f| {
                    let init = get_init_code(f);
                    let name = &f.ident;
                    quote! { #name: #init }
                });
                let names: Vec<_> = fields
                    .named
                    .iter()
                    .map(|f| f.ident.as_ref().unwrap())
                    .collect();
                quote! {
                    #name::#vname { #( ref #names ),* } => {
                        #name::#vname {
                            #(#defaults),*
                        }
                    }
                }
            }

            Fields::Unnamed(fields) => {
                let defaults = fields.unnamed.iter().map(get_init_code);
                let bindings: Vec<_> = (0..fields.unnamed.len())
                    .map(|i| syn::Ident::new(&format!("f{i}"), proc_macro2::Span::call_site()))
                    .collect();
                quote! {
                    #name::#vname( #( ref #bindings ),* ) => {
                        #name::#vname(
                            #(#defaults),*
                        )
                    }
                }
            }

            Fields::Unit => {
                quote! { #name::#vname => #name::#vname }
            }
        }
    });

    quote! {
        match self {
            #(#arms),*
        }
    }
}

fn get_init_code(field: &Field) -> proc_macro2::TokenStream {
    field
        .attrs
        .iter()
        .filter_map(|attr| match &attr.meta {
            syn::Meta::List(meta_list) => {
                let path = meta_list.path.segments.last().unwrap();
                let name = path.ident.to_string();
                if name != "init" {
                    return None;
                }
                let imp = &meta_list.tokens;
                Some(quote! {#imp.into()})
            }
            _ => None,
        })
        .next()
        .unwrap_or(quote! { Default::default() })
}
