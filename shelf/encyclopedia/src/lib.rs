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
    bincode::Error,
    Schema(Box<dyn std::error::Error + 'static>),
    Index(Box<dyn std::error::Error + 'static>),
);

use puzzle::Puzzle;
use std::collections::HashMap;

#[test]
fn test_search() {
    // TODO:

    let mut document = HashMap::new();

    document.insert("title", "The Old Man and the Sea");
    document.insert(
        "body",
        "He was an old man who fished alone in a skiff in the Gulf Stream and \
he had gone eighty-four days now without taking a fish.",
    );

    let mut key = bincode::serialize(&document).unwrap();

    let mut puzzle = Puzzle::empty();

    let key = puzzle.wrap(key);

    let context = context(&puzzle).unwrap();

    write(&puzzle, &context).unwrap();

    //let got = read(&context, "sea whale").unwrap();

    //dbg!(got);
}

use serde::Serialize;
use tantivy::{
    collector::TopDocs,
    query::QueryParser,
    schema::{Document, Field, Schema, STORED, TEXT},
    DocAddress, Index, ReloadPolicy, Score,
};

struct Context {
    schema: Schema,
    fields: HashMap<String, Field>,
    index: Index,
}

fn context(puzzle: &Puzzle) -> Result<Context, Error> {
    let (schema, fields) = schema(puzzle)?;
    let index = index(&schema)?;

    Ok(Context {
        schema,
        fields,
        index,
    })
}

fn write(puzzle: &Puzzle, context: &Context) -> Result<(), Error> {
    let mut index_writer = context.index.writer(100_000_000)?;

    for key in puzzle.keys() {
        let map = bincode::deserialize::<HashMap<String, &str>>(&key)?;
        let mut document = Document::default();

        for (key, value) in map {
            let field = context.fields.get(&key).unwrap();
            document.add_text(*field, value);
        }

        index_writer.add_document(document);
    }

    index_writer.commit()?;

    Ok(())
}

fn schema(puzzle: &Puzzle) -> Result<(Schema, HashMap<String, Field>), Error> {
    let mut schema_builder = Schema::builder();

    let mut fields = HashMap::new();

    for key in puzzle.keys() {
        let map = bincode::deserialize::<HashMap<_, Vec<u8>>>(&key)?;

        for (key, _) in map {
            let field = schema_builder.add_text_field(key, TEXT | STORED);
            fields.insert(key.into(), field);
        }
    }

    let schema = schema_builder.build();

    Ok((schema, fields))
}

fn index(schema: &Schema) -> Result<Index, Error> {
    let target = painting::frame()?;

    Index::create_in_dir(target.join("encyclopedia"), schema.clone()).map_err(Error::from)
}

//fn read(context: &Context, input: &str) -> Result<impl std::fmt::Debug, Error> {
//let reader = context.index.reader()?;

//let searcher = reader.searcher();

//let query_parser =
//QueryParser::for_index(&context.index, context.fields.values().cloned().collect());

//let query = query_parser.parse_query(input)?;

//let top_docs: Vec<(Score, DocAddress)> = searcher.search(&query, &TopDocs::with_limit(10))?;

//let mut output = vec![];

//for (_score, key) in top_docs {
//let found = searcher.doc(key)?;
//output.push(context.schema.to_json(&found));
//}

//Ok(output)
//}
