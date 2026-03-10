use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemStruct};

pub fn wasm_getters_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
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
