book::error!(
    Incomplete,
    std::io::Error,
    ExpectedField,
    painting::Error,
    tantivy::TantivyError,
    SchemaRead,
    NoField(String),
    Other(Box<dyn std::error::Error + 'static>),
    tantivy::query::QueryParserError,
);

#[test]
fn test_search() {
    // TODO:
}

use puzzle::Puzzle;
use serde::Serialize;
use tantivy::{
    collector::TopDocs,
    query::QueryParser,
    schema::{Document, Field, Schema, STORED, TEXT},
    DocAddress, Index, ReloadPolicy, Score,
};

pub fn write(puzzle: &Puzzle) -> Result<impl Serialize, Error> {
    let index = index()?;

    let mut index_writer = index.writer(100_000_000)?;

    let (schema, fields) = schema();

    for next in puzzle.keys() {
        let mut document = Document::default();
        let data = vec![
            next.as_ref(),
            "The Old Man and the Sea",
            "He was an old man who fished alone in a skiff in the Gulf Stream and \
        he had gone eighty-four days now without taking a fish.",
        ];

        for (key, field) in fields.iter().enumerate() {
            document.add_text(*field, data[key]);
        }

        index_writer.add_document(document);
    }

    index_writer.commit()?;

    Ok(format!("{:?}", puzzle))
}

use once_cell::sync::Lazy;

static SCHEMA: Lazy<(Schema, Vec<Field>)> = Lazy::new(|| {
    let mut schema_builder = Schema::builder();

    let key = schema_builder.add_text_field("key", TEXT | STORED);
    let title = schema_builder.add_text_field("title", TEXT | STORED);
    let body = schema_builder.add_text_field("body", TEXT);

    let schema = schema_builder.build();

    (schema, vec![key, title, body])
});

fn schema() -> &'static (Schema, Vec<Field>) {
    Lazy::force(&SCHEMA)
}

fn field(schema: &Schema, key: &str) -> Result<Field, Error> {
    schema
        .get_field(key)
        .map_or_else(|| Err(Error::NoField(key.into())), Ok)
}

#[derive(Debug, thiserror::Error)]
enum IndexError {
    #[error("Index {0}")]
    Index(#[from] tantivy::TantivyError),
    #[error("Painting {0}")]
    Painting(#[from] painting::Error),
}

static INDEX: Lazy<Result<Index, IndexError>> = Lazy::new(|| {
    let (schema, _) = schema();

    let target = painting::frame()?;

    Index::create_in_dir(target, schema.clone()).map_err(IndexError::from)
});

fn index<'a>() -> Result<&'a Index, Error> {
    match Lazy::force(&INDEX) {
        &Ok(ref index) => Ok(index),
        &Err(ref error) => Err(Error::Other(Box::new(error))),
    }
}

pub fn read(context: &Puzzle) -> Result<impl Serialize, Error> {
    let (schema, fields) = schema();
    let index = index()?;
    let reader = index.reader()?;

    let searcher = reader.searcher();

    let query_parser = QueryParser::for_index(&index, fields.to_owned());

    let query = query_parser.parse_query("sea whale")?;

    let top_docs: Vec<(Score, DocAddress)> = searcher.search(&query, &TopDocs::with_limit(10))?;

    let mut output = vec![];

    for (_score, key) in top_docs {
        let found = searcher.doc(key)?;
        output.push(schema.to_json(&found));
    }

    Ok(output)
}
