use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(ch.to_ascii_lowercase());
        } else {
            result.push(ch);
        }
    }
    result
}

#[proc_macro_derive(EnuMap)]
pub fn derive_enumap(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let enum_name = &input.ident;
    let map_name = syn::Ident::new(&format!("{enum_name}Map"), enum_name.span());

    let Data::Enum(data_enum) = &input.data else {
        return syn::Error::new_spanned(&input.ident, "EnuMap can only be derived for enums")
            .to_compile_error()
            .into();
    };

    // Extract variant names (only support unit variants for now)
    let mut variant_names = Vec::new();
    let mut field_names = Vec::new();
    for variant in &data_enum.variants {
        match &variant.fields {
            Fields::Unit => {
                variant_names.push(&variant.ident);
                // Convert to snake_case for the field name
                let field_name = syn::Ident::new(
                    &to_snake_case(&variant.ident.to_string()),
                    variant.ident.span(),
                );
                field_names.push(field_name);
            }
            _ => {
                return syn::Error::new_spanned(
                    variant,
                    "EnuMap only supports unit variants (variants without fields)",
                )
                .to_compile_error()
                .into();
            }
        }
    }

    // Generate the struct fields
    let struct_fields = field_names.iter().map(|name| {
        quote! {
            pub #name: T
        }
    });

    let expanded = quote! {
        pub struct #map_name<T> {
            #(#struct_fields),*
        }

        impl<T> #map_name<T> {
            pub fn get(&self, key: &#enum_name) -> &T {
                match key {
                    #(#enum_name::#variant_names => &self.#field_names),*
                }
            }

            pub fn get_mut(&mut self, key: &#enum_name) -> &mut T {
                match key {
                    #(#enum_name::#variant_names => &mut self.#field_names),*
                }
            }
        }
    };

    TokenStream::from(expanded)
}
