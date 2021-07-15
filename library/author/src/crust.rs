/// earth's crust
use proc_macro2::TokenStream;
use quote::quote;

// Use this to get workspace root:
//
// cargo metadata --format-version=1 | jq .workspace_root
//

// fn macro_for_workspace_root_path
pub fn workspace_root(input: TokenStream) -> Result<TokenStream, Error> {
    let input = syn::parse2::<Kinds>(input)?;

    Ok(())
}
