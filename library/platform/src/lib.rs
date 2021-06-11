use std::collections::HashMap;

use proc_macro::TokenStream;
use quote::quote;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Platform {
    features: HashMap<String, Vec<String>>,
}

#[proc_macro]
pub fn build(input: TokenStream) -> TokenStream {
    let configuration = include_str!("../../../Cargo.toml");

    let platform: Platform = match toml::from_str(configuration) {
        Ok(platform) => platform,
        Err(error) => {
            compile_error!("Unable to parse Cargo.toml")
        }
    };

    println!("{:?}", platform);
    println!("{:?}", input);

    let mut body = quote!();

    for (feature, crates) in platform.features {
        for handler in crates {
            body = quote! {
                #body

                #[cfg(feature = stringify!(#feature))]
                {
                    match #handler::handle(&mut input) {
                        Err(error) => match &error {
                            #handler::Error::Incomplete => {}
                            _ => {
                                eprintln!("error handling {} input: {}", stringify!(#handler), error);
                            }
                        },
                        _ => {}
                    };
                }
            };
        }
    }

    TokenStream::from(body)
}
