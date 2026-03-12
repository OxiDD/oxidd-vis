use heck::{ToSnekCase, ToTitleCase, ToTrainCase};
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, spanned::Spanned, Data, DeriveInput, Field, Fields, Type, TypePath};

pub fn derive_component_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let label = get_label(&input);
    let name = input.ident;
    let body = match input.data {
        Data::Struct(data) => component_struct(&data.fields, &label),
        Data::Enum(data) => component_enum(&name, &data.variants, &label),
        _ => panic!("Can only derive Component for structs and enums"),
    };

    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics Into<crate::new_wasm_interface::Component> for #name #ty_generics #where_clause {
            fn into(self) -> crate::new_wasm_interface::Component {
                #body
            }
        }
    };

    expanded.into()
}

struct LabelData {
    should_label: bool,
    comp_map: proc_macro2::TokenStream,
    convert_text: bool,
}
fn get_label(input: &DeriveInput) -> LabelData {
    let mut should_label = false;
    let mut comp_map = quote! {
        |name, val| crate::components::label_component::LabelComp::wrapped(name, val)
    };
    let mut convert_text = false;
    for attr in &input.attrs {
        if !attr.path().is_ident("label") {
            continue;
        }
        should_label = true;
        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("map") {
                let map: syn::Expr = meta.value()?.parse()?;

                comp_map = quote! {
                    |name, val| crate::inputs::wrapper::DynWrappedInput::new(
                        val,
                        move |comp| (#map)(name, comp),
                        true
                    )
                };
            }
            if meta.path.is_ident("spaced") {
                convert_text = true;
            }

            Ok(())
        });
    }
    LabelData {
        should_label,
        comp_map,
        convert_text,
    }
}

fn component_struct(fields: &syn::Fields, label: &LabelData) -> proc_macro2::TokenStream {
    match fields {
        Fields::Named(fields) => {
            let comps = fields.named.iter().map(|f| {
                let name = &f.ident;
                get_comp_code(f, quote! {self.#name}, label)
            });
            quote! { ( #(#comps),* ).into() }
        }
        Fields::Unnamed(fields) => {
            let comps = fields.unnamed.iter().enumerate().map(|(i, f)| {
                let idx = syn::Index::from(i);
                get_comp_code(f, quote! {self.#idx}, label)
            });
            quote! { ( #(#comps),* ).into() }
        }
        Fields::Unit => quote! { Self },
    }
}
fn component_enum(
    name: &syn::Ident,
    variants: &syn::punctuated::Punctuated<syn::Variant, syn::token::Comma>,
    label: &LabelData,
) -> proc_macro2::TokenStream {
    let arms = variants.iter().map(|variant| {
        let vname = &variant.ident;

        match &variant.fields {
            Fields::Named(fields) => {
                let comps = fields.named.iter().map(|f| {
                    let name = &f.ident;
                    get_comp_code(f, quote! {self.#name}, label)
                });
                let names: Vec<_> = fields
                    .named
                    .iter()
                    .map(|f| f.ident.as_ref().unwrap())
                    .collect();
                quote! {
                    #name::#vname { #( ref #names ),* } => ( #(#comps),* ).into()
                }
            }

            Fields::Unnamed(fields) => {
                let comps = fields.unnamed.iter().enumerate().map(|(i, f)| {
                    let idx = syn::Index::from(i);
                    get_comp_code(f, quote! {self.#idx}, label)
                });
                let bindings: Vec<_> = (0..fields.unnamed.len())
                    .map(|i| syn::Ident::new(&format!("f{i}"), proc_macro2::Span::call_site()))
                    .collect();
                quote! {
                    #name::#vname( #( ref #bindings ),* ) => ( #(#comps),* ).into()
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

fn get_comp_code(
    field: &Field,
    val: proc_macro2::TokenStream,
    label: &LabelData,
) -> proc_macro2::TokenStream {
    let ident = &field.ident;
    let mut name = ident.as_ref().unwrap().to_string();
    if label.convert_text {
        name = name.to_title_case();
    }
    let mut name = string_expr(&name, ident.span());
    let mut should_label = label.should_label;
    let mut comp_map = label.comp_map.clone();
    for attr in &field.attrs {
        if attr.path().is_ident("label") {
            should_label = true;
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("map") {
                    let map: syn::Expr = meta.value()?.parse()?;

                    comp_map = quote! {
                        |name, val| crate::inputs::wrapper::DynWrappedInput::new(
                            val,
                            move |comp| (#map)(name, comp),
                            true
                        )
                    };
                }
                if meta.path.is_ident("text") {
                    name = meta.value()?.parse()?;
                }

                Ok(())
            });
        }
    }
    if should_label {
        quote! { (#comp_map)(#name, #val)  }
    } else {
        quote! { #val }
    }
}

fn string_expr(s: &str, span: Span) -> syn::Expr {
    syn::Expr::Lit(syn::ExprLit {
        attrs: vec![],
        lit: syn::Lit::Str(syn::LitStr::new(s, span)),
    })
}
