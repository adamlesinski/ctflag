extern crate proc_macro;
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;
use quote::quote;

fn generate_field_parsing(field: &syn::Field) -> TokenStream {
    TokenStream::from(quote! {
        {}
    })
}

#[proc_macro_derive(Flags)]
pub fn flag_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);
    let name = &ast.ident;

    if let syn::Data::Struct(data) = ast.data {
        if let syn::Fields::Named(fields) = data.fields {
            let field_parsing = fields.named.iter().map(generate_field_parsing);
            let expanded = quote! {
                impl rarg::Flags for #name {
                    fn from_args(_: std::env::Args) -> Result<Self, rarg::ParseError> {
                        //#(#field_parsing)*
                        Err(rarg::ParseError::FlagError("unimplemented".to_owned()))
                    }

                    fn description() -> String {
                        "".to_owned()
                    }
                }
            };
            return TokenStream::from(expanded);
        }
    }
    panic!("Bad things!");
}
