/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use proc_macro2::{Span, TokenStream};
use quote::quote;

#[derive(Eq, PartialEq, Debug)]
pub struct GeneratedType {
    pub src: String,
    pub name: String,
    pub properties: Vec<GeneratedProperty>,
}

impl Into<TokenStream> for GeneratedType {
    fn into(self) -> TokenStream {
        let GeneratedType {
            src,
            name,
            properties,
        } = self;

        let properties: Vec<TokenStream> = properties.into_iter().map(|x| x.into()).collect();

        let comment = format!("///Generated from {}", src)
            .parse::<TokenStream>()
            .unwrap();

        let name = proc_macro2::Ident::new(&name, Span::call_site());

        quote! {
            #comment
            #[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
            pub struct #name {
                #(#properties),*
            }
        }
    }
}

#[derive(Eq, PartialEq, Debug)]
pub struct GeneratedProperty {
    pub name: String,
    pub property_type: String,
    pub serde_options: SerdeOptions,
}

impl Into<TokenStream> for GeneratedProperty {
    fn into(self) -> TokenStream {
        let GeneratedProperty {
            name,
            property_type,
            serde_options,
        } = self;

        let mut attributes: Vec<TokenStream> = Vec::new();

        match serde_options.rename {
            Some(name) => {
                attributes.push(quote! {
                    #[serde(rename = #name)]
                });
            }
            None => {}
        };

        match serde_options.skip_serializing_if {
            Some(option) => {
                attributes.push(quote! {
                    #[serde(skip_serializing_if = #option)]
                });
            }
            None => {}
        };

        let name = proc_macro2::Ident::new(&name, Span::call_site());
        let property_type = property_type.parse::<TokenStream>().unwrap();

        quote! {
            #(#attributes)*
            pub #name: #property_type
        }
    }
}

#[derive(Eq, PartialEq, Debug)]
pub struct SerdeOptions {
    pub rename: Option<String>,
    pub skip_serializing_if: Option<String>,
}

#[cfg(test)]
mod generated_tests {
    use crate::generated::{GeneratedProperty, GeneratedType, SerdeOptions};
    use proc_macro2::TokenStream;

    #[test]
    fn should_generate_valid_property_rust_code() {
        let tokens: TokenStream = create_property().into();

        assert_eq!(
            tokens.to_string(),
            String::from("# [serde (rename = \"original name\")] pub new_name : String")
        )
    }

    #[test]
    fn should_generate_valid_struct_rust_code() {
        let struct_type = GeneratedType {
            src: String::from("nirvana"),
            name: String::from("new_name"),
            properties: vec![create_property(), create_property()],
        };

        let tokens: TokenStream = struct_type.into();

        assert_eq!(
            tokens.to_string(),
            String::from("# [doc = \"Generated from nirvana\"] # [derive (Clone , PartialEq , Debug , Deserialize , Serialize)] pub struct new_name { # [serde (rename = \"original name\")] pub new_name : String , # [serde (rename = \"original name\")] pub new_name : String }")
        )
    }

    fn create_property() -> GeneratedProperty {
        GeneratedProperty {
            name: String::from("new_name"),
            property_type: String::from("String"),
            serde_options: SerdeOptions {
                rename: Some(String::from("original name")),
                skip_serializing_if: None,
            },
        }
    }
}
