use proc_macro::TokenStream;
use proc_macro2::{Literal, Span};
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    parse2, parse_macro_input, Expr, Field, GenericArgument, Ident, Item, ItemStruct, Result,
    Token, Type,
};

#[proc_macro_attribute]
pub fn watchable_setters(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemStruct);

    for field in input.fields.iter_mut() {
        let syn::Type::Path(type_path) = &field.ty else {
            continue;
        };
        let main_type = type_path.path.segments.last().unwrap();
        let type_ident = &main_type.ident;

        let mut arg = None;
        field.attrs.retain(|attr| match &attr.meta {
            syn::Meta::List(meta_list) => {
                let path = meta_list.path.segments.last().unwrap();
                let name = path.ident.to_string();
                if name != "setter" {
                    return true;
                }
                arg = Some(meta_list.tokens.clone());
                false
            }
            _ => true,
        });
        let Some(arg) = arg else {
            continue;
        };

        let Ok(setter_type) = parse2::<TypeMaybeExpr>(arg) else {
            continue;
        };
        let value_type = setter_type.ty;
        field.attrs.push(
            syn::parse_quote!(#[builder(with = |val: impl crate::util::watchables::IntoWatchable<#value_type> + 'static| #type_ident::new(val))]),
        );

        // If a default is provided, add the default
        if let Some(def) = setter_type.expr {
            field
                .attrs
                .push(syn::parse_quote!(#[builder(default=#type_ident::new(crate::util::watchables::IntoWatchable::<#value_type>::into_watchable(#def)))]));
            continue;
        }

        // Add default init for options
        let syn::Type::Path(type_path) = value_type else {
            continue;
        };
        let setter_type = type_path.path.segments.last().unwrap();
        if setter_type.ident.to_string() != "Option" {
            continue;
        }
        field
            .attrs
            .push(syn::parse_quote!(#[builder(default=#type_ident::new(crate::util::watchables::Constant::new(None)))]));
    }

    TokenStream::from(quote! {
       #input
    })
}

/// Represents `Type [, Expr]`
struct TypeMaybeExpr {
    ty: Type,
    comma: Option<Token![,]>,
    expr: Option<Expr>,
}

impl Parse for TypeMaybeExpr {
    fn parse(input: ParseStream) -> Result<Self> {
        let ty: Type = input.parse()?;
        let mut comma = None;
        let mut expr = None;

        if input.peek(Token![,]) {
            comma = Some(input.parse()?);
            expr = Some(input.parse()?);
        }

        Ok(TypeMaybeExpr { ty, comma, expr })
    }
}

#[proc_macro_attribute]
pub fn wasm_getters(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemStruct);

    let struct_ident = &input.ident;
    let mut setters = Vec::new();
    for field in input.fields.iter_mut() {
        let Some(field_name) = &field.ident else {
            continue;
        };
        let syn::Type::Path(type_path) = &field.ty else {
            continue;
        };
        let main_type = type_path.path.segments.last().unwrap();
        let type_ident = &main_type.ident;

        let mut is_getter = false;
        field.attrs.retain(|attr| match &attr.meta {
            syn::Meta::Path(p) => match p.get_ident() {
                Some(v) => {
                    if v.to_string() == "getter" {
                        is_getter = true;
                        false
                    } else {
                        true
                    }
                }
                None => true,
            },
            _ => true,
        });
        if !is_getter {
            continue;
        }

        let func: syn::ItemImpl = syn::parse_quote!(
            #[wasm_bindgen]
            impl #struct_ident {
                #[wasm_bindgen(getter)]
                pub fn #field_name(&self) -> #type_ident {
                    self.#field_name.clone()
                }
            }
        );
        setters.push(func)
    }

    TokenStream::from(quote! {
        #input
        #(#setters)*
    })
}

#[proc_macro]
pub fn gen_tuple_into_component_vec_watchables(data: TokenStream) -> TokenStream {
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
// #[proc_macro_attribute]
// pub fn watchable_getters(_attr: TokenStream, item: TokenStream) -> TokenStream {
//     let mut input = parse_macro_input!(item as ItemStruct);

//     let mut getters = Vec::new();
//     let struct_ident = &input.ident;
//     for field in input.fields.iter_mut() {
//         let syn::Type::Path(type_path) = &field.ty else {
//             continue;
//         };

//         let main_type = type_path.path.segments.last().unwrap();
//         let type_ident = &main_type.ident;
//         let type_name = type_ident.to_string();
//         if !type_name.starts_with("Watchable") {
//             continue;
//         }
//         if !main_type.arguments.is_none() {
//             continue;
//         }

//         let Some(field_name) = &field.ident else {
//             continue;
//         };

//         let old_size = field.attrs.len();
//         field.attrs.retain(|attr| !attr.path().is_ident("getter"));
//         let is_getter = old_size != field.attrs.len();
//         if !is_getter {
//             continue;
//         }

//         field.attrs.push(syn::parse_quote!(#[wasm_bindgen(skip)]));
//         let func: syn::ItemImpl = syn::parse_quote!(
//             #[wasm_bindgen]
//             impl #struct_ident {
//                 #[wasm_bindgen(getter)]
//                 pub fn #field_name(&self) -> #type_ident {
//                     self.#field_name.clone()
//                 }
//             }
//         );
//         getters.push(func);
//     }

//     TokenStream::from(quote! {
//         #input
//         #(#getters)*
//     })
// }
