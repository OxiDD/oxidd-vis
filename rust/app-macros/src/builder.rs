use heck::ToSnakeCase;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    parse2, parse_macro_input, parse_quote, Expr, ItemStruct, Result, Token, Type,
};

pub fn builder_into_comp_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemStruct);

    input.attrs.push(parse_quote!(#[builder(derive(Into))]));
    let struct_ident = &input.ident;
    let builder_struct_ident = format_ident!("{}Builder", struct_ident);
    let builder_path = format_ident!("{}", builder_struct_ident.to_string().to_snake_case());
    let builder_path = quote!(self::#builder_path);
    let is_complete_path = quote!(#builder_path::IsComplete);
    let builder_struct_path = quote!(self::#builder_struct_ident);

    TokenStream::from(quote! {
        #input
        impl<S: #is_complete_path> Into<Component> for #builder_struct_path<S> {
            fn into(self) -> Component {
                self.build().into()
            }
        }
    })
}

pub fn watchable_setters_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
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
