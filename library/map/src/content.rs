author::error!(
    Incomplete,
    std::io::Error,
    NoParent(String),
    ignore::Error,
    std::path::StripPrefixError,
    context::Error,
    std::string::FromUtf8Error,
    std::str::Utf8Error,
    tinytemplate::error::Error,
);

use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Write},
    path::Path,
    str,
};

pub fn path<'a>(
    path: &'a Path,
    variables: &'a HashMap<&'a str, serde_json::Value>,
) -> Result<Vec<u8>, Error> {
    if path.is_file() {
        file(path, variables)
    } else {
        directory(path, variables)
    }
}

fn file<'a>(
    path: &'a Path,
    variables: &'a HashMap<&'a str, serde_json::Value>,
) -> Result<Vec<u8>, Error> {
    let mut file = File::open(path)?;

    let mut output = vec![];

    file.read_to_end(&mut output)?;

    if let Ok(output) = std::str::from_utf8(&output) {
        return document(output, variables);
    }

    Ok(output)
}

fn directory<'a>(
    directory: &'a Path,
    variables: &'a HashMap<&'a str, serde_json::Value>,
) -> Result<Vec<u8>, Error> {
    let context = directory.file_stem();

    let mut output = vec![];

    write!(
        &mut output,
        "<details>\n\
            <summary>\n\
                Directory: {}\n\
            </summary>\n\n",
        directory.display()
    )?;

    let walker = ignore::WalkBuilder::new(directory)
        .hidden(false)
        .max_depth(Some(1))
        .build();

    let mut buffer = vec![];

    for entry in walker {
        let entry = entry?;

        let path = entry.path();

        if path == directory {
            continue;
        }

        if context == path.file_name() {
            let mut file = File::open(&path)?;
            file.read_to_end(&mut buffer)?;
        }

        let parent = match path.parent() {
            Some(parent) => parent,
            None => return Err(Error::NoParent(path.display().to_string())),
        };

        let title = path.strip_prefix(parent)?;

        let cleaned = path.strip_prefix(context::home()?)?;

        writeln!(
            &mut output,
            "- [{}]({})",
            title.display(),
            cleaned.display()
        )?;
    }

    output.write(b"</details>\n\n")?;
    output.write(&buffer)?;

    document(&str::from_utf8(&output)?, variables)
}

fn template<'a>(
    input: &'a str,
    variables: &'a HashMap<&'a str, serde_json::Value>,
) -> Result<Vec<u8>, Error> {
    use pulldown_cmark::{html::write_html, Parser};
    use tinytemplate::TinyTemplate as Templates;

    let mut templates = Templates::new();

    templates.add_template("document", input)?;

    let buffer = templates.render("document", variables)?;

    let parser = Parser::new(&buffer);

    let mut output = vec![];

    write_html(&mut output, parser)?;

    Ok(output)
}

static DOCUMENT: &'static str = include_str!("../../console/console.html");

fn document<'a>(
    input: &'a str,
    variables: &'a HashMap<&'a str, serde_json::Value>,
) -> Result<Vec<u8>, Error> {
    println!("{}", DOCUMENT);

    let mut output = vec![];

    write!(&mut output, "<!doctype html><html><body>")?;

    output.write(&template(input, variables)?)?;

    write!(&mut output, "</body></html>")?;

    Ok(output)
}
