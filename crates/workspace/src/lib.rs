#[proc_macro]
pub fn crate_names(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match read_crate_names(input.into()) {
        Ok(tokens) => tokens.into(),
        Err(diag) => diag.emit_as_expr_tokens().into(),
    }
}

use ::{
    glob::glob,
    proc_macro2::TokenStream,
    proc_macro2_diagnostics::{Diagnostic, Level},
    quote::quote,
    serde::Deserialize,
    std::{fs::File, io::Read},
};

#[derive(Deserialize)]
struct CargoToml {
    workspace: Workspace,
}

#[derive(Deserialize)]
struct Workspace {
    members: Vec<String>,
}

fn read_crate_names(_: TokenStream) -> Result<TokenStream, Diagnostic> {
    let root = environment::workspace().map_err(|error| {
        Diagnostic::new(
            Level::Error,
            format!("error determining workspace root: {}", error),
        )
    })?;

    let cargo_toml: CargoToml = File::open(root.join("Cargo.toml"))
        .map_err(|error| {
            Diagnostic::new(Level::Error, format!("error opening Cargo.toml: {}", error))
        })
        .and_then(|mut file| {
            let mut buffer = vec![];

            file.read_to_end(&mut buffer).map_err(|error| {
                Diagnostic::new(Level::Error, format!("error reading Cargo.toml: {}", error))
            })?;

            Ok(buffer)
        })
        .and_then(|buffer| {
            toml::de::from_slice(&buffer).map_err(|error| {
                Diagnostic::new(Level::Error, format!("error parsing Cargo.toml: {}", error))
            })
        })?;

    let mut crate_names = vec![];

    for entry in cargo_toml.workspace.members {
        dbg!(&entry);

        let based = root.join(entry);

        let paths = glob(&format!("{}", based.display())).map_err(|error| {
            Diagnostic::new(
                Level::Error,
                format!("error building matcher from workspace members: {}", error),
            )
        })?;

        for entry in paths {
            dbg!(&entry);

            let member = entry.map_err(|error| {
                Diagnostic::new(
                    Level::Error,
                    format!("invalid glob from workspace member: {}", error),
                )
            })?;

            let name = member
                .file_name()
                .ok_or(Diagnostic::new(Level::Error, "member has no name"))?;

            let name = name.to_str().ok_or(Diagnostic::new(
                Level::Error,
                "member name is not valid utf8",
            ))?;

            crate_names.push(name.to_string());
        }
    }

    let filtered = crate_names
        .iter()
        .filter(|name| !name.ends_with("workspace"));

    Ok(quote! { &[#(#filtered,)*] })
}
