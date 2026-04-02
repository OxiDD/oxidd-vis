use darling::{util::Flag, FromDeriveInput, FromField, FromMeta};
use heck::{ToSnekCase, ToTitleCase, ToTrainCase};
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, spanned::Spanned, Data, DeriveInput, Field, Fields, Type, TypePath};

pub fn derive_component_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match derive_component_impl_internal(input) {
        Ok(tokens) => tokens.into(),
        Err(e) => e.write_errors().into(),
    }
}
pub fn derive_component_impl_internal(input: DeriveInput) -> darling::Result<TokenStream> {
    let label_options = LabelOptions::from_derive_input(&input)?;
    let comp_options = CompOptions::from_derive_input(&input)?;
    let name = input.ident;
    let body = match input.data {
        Data::Struct(data) => component_struct(&data.fields, &label_options)?,
        Data::Enum(data) => component_enum(&name, &data.variants, &label_options)?,
        _ => panic!("Can only derive Component for structs and enums"),
    };

    let body = match (comp_options.map, comp_options.build) {
        (Some(map), _) => quote! {
            let comp_vec: crate::components::composite_component::ComponentVecWatchable = #body;
            (#map)(comp_vec).into()
        },
        (_, Some(build)) => quote! {
            {
                let builder = crate::components::composite_component::CompositeComp::builder();
                let comp_vec: crate::components::composite_component::ComponentVecWatchable = #body;
                #build.build(comp_vec).into()
            }
        },
        _ => body,
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

    Ok(expanded.into())
}

#[derive(FromField)]
#[darling(attributes(label))]
struct FieldLabelOptions {
    enabled: Option<bool>,
    map: Option<syn::Expr>,
    text: Option<syn::LitStr>,
    spaced: Option<bool>,
}

#[derive(FromDeriveInput)]
#[darling(attributes(label))]
struct LabelOptions {
    enabled: Option<bool>,
    map: Option<syn::Expr>,
    spaced: Option<bool>,
}

#[derive(FromField)]
#[darling(attributes(comp))]
struct FieldCompOptions {
    map: Option<syn::Expr>,
    build: Option<syn::Expr>,
}

#[derive(FromDeriveInput)]
#[darling(attributes(comp))]
struct CompOptions {
    map: Option<syn::Expr>,
    build: Option<syn::Expr>,
}

fn component_struct(
    fields: &syn::Fields,
    label: &LabelOptions,
) -> darling::Result<proc_macro2::TokenStream> {
    let res = match fields {
        Fields::Named(fields) => {
            let comps = fields
                .named
                .iter()
                .map(|f| {
                    let name = &f.ident;
                    get_comp_code(f, quote! {self.#name}, label)
                })
                .collect::<Result<Vec<_>, _>>()?;
            quote! { ( #(#comps),* ).into() }
        }
        Fields::Unnamed(fields) => {
            let comps = fields
                .unnamed
                .iter()
                .enumerate()
                .map(|(i, f)| {
                    let idx = syn::Index::from(i);
                    get_comp_code(f, quote! {self.#idx}, label)
                })
                .collect::<Result<Vec<_>, _>>()?;
            quote! { ( #(#comps),* ).into() }
        }
        Fields::Unit => quote! { Self },
    };
    Ok(res)
}
fn component_enum(
    name: &syn::Ident,
    variants: &syn::punctuated::Punctuated<syn::Variant, syn::token::Comma>,
    label: &LabelOptions,
) -> darling::Result<proc_macro2::TokenStream> {
    let arms = variants
        .iter()
        .map(|variant| -> darling::Result<_> {
            let vname = &variant.ident;

            let arm = match &variant.fields {
                Fields::Named(fields) => {
                    let comps = fields
                        .named
                        .iter()
                        .map(|f| {
                            let name = &f.ident;
                            get_comp_code(f, quote! {self.#name}, label)
                        })
                        .collect::<Result<Vec<_>, _>>()?;
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
                    let comps = fields
                        .unnamed
                        .iter()
                        .enumerate()
                        .map(|(i, f)| {
                            let idx = syn::Index::from(i);
                            get_comp_code(f, quote! {self.#idx}, label)
                        })
                        .collect::<Result<Vec<_>, _>>()?;
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
            };
            Ok(arm)
        })
        .collect::<Result<Vec<_>, _>>()?;

    let res = quote! {
        match self {
            #(#arms),*
        }
    };
    Ok(res)
}

fn get_comp_code(
    field: &Field,
    val: proc_macro2::TokenStream,
    label: &LabelOptions,
) -> darling::Result<proc_macro2::TokenStream> {
    let input_data = get_labeled_input_data(field, val, label)?;
    get_input_comp(field, input_data)
}

fn get_labeled_input_data(
    field: &Field,
    val: proc_macro2::TokenStream,
    label: &LabelOptions,
) -> darling::Result<proc_macro2::TokenStream> {
    let field_label = FieldLabelOptions::from_field(&field)?;

    let should_label = field_label.enabled.or(label.enabled).unwrap_or(
        label.map.is_some()
            || label.spaced.is_some()
            || field_label.map.is_some()
            || field_label.text.is_some(),
    );
    if !should_label {
        return Ok(val);
    }

    let name = field_label.text.clone().unwrap_or_else(|| {
        let ident = &field.ident;
        let mut name = ident.as_ref().unwrap().to_string();
        if field_label.spaced.or(label.spaced).unwrap_or(false) {
            name = name.to_title_case();
        }
        syn::LitStr::new(&name, ident.span())
    });
    let comp_map = field_label
        .map
        .as_ref()
        .or(label.map.as_ref())
        .map(|map| {
            quote! {
                |name, val| crate::inputs::wrapper::DynWrappedInput::new(
                    val,
                    move |comp| (#map)(name, comp),
                    true
                )
            }
        })
        .unwrap_or(quote! {
            |name, val| crate::components::label_component::LabelComp::wrapped(name, val)
        });
    Ok(quote! { (#comp_map)(#name, #val)  })
}

fn get_input_comp(
    field: &Field,
    val: proc_macro2::TokenStream,
) -> darling::Result<proc_macro2::TokenStream> {
    let field_comp = FieldCompOptions::from_field(&field)?;
    let field_type = &field.ty;
    let res = match (field_comp.map, field_comp.build) {
        (Some(map), _) => quote! {
            (#map)(#val)
        },
        (_, Some(build)) =>
        // quote! {
        //     (#build)(<#field_type as crate::inputs::wrapper::DefaultInputComp>::Comp::builder(#val))
        // },
        {
            quote! {
                {
                    let builder = <#field_type as crate::inputs::wrapper::DefaultInputComp>::Comp::builder(#val);
                    #build
                }
            }
        }
        _ => val,
    };
    Ok(res)
}
