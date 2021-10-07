#[test]
fn test_search() {
    let mut puzzle = Puzzle::new();
    puzzle.wrap(
        "The Old Man and the Sea",
        "He was an old man who fished alone in a skiff in the Gulf Stream and \
he had gone eighty-four days now without taking a fish.",
    );
    let got = puzzle.search("was an").unwrap();
    dbg!(got);
    assert!(false);
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Painting {0}")]
    Painting(#[from] painting::Error),
    #[error("Index {0}")]
    Index(#[from] tantivy::TantivyError),
    #[error("Query {0}")]
    Query(#[from] tantivy::query::QueryParserError),
    #[error("Machine {0}")]
    Machine(#[from] std::io::Error),
    #[error("IndexTarget {0}")]
    IndexTarget(#[from] tantivy::directory::error::OpenDirectoryError),
}

use crdts::{merkle_reg::MerkleReg, CmRDT, CvRDT};

#[derive(Debug)]
pub struct Puzzle {
    content: MerkleReg<Pair>,
}

#[derive(Debug)]
pub struct Pair(String, String);

impl Pair {
    fn bytes(&self) -> Vec<u8> {
        [self.0.as_bytes(), self.1.as_bytes()].concat()
    }
}

use tantivy::{
    collector::TopDocs,
    directory::MmapDirectory,
    query::QueryParser,
    schema::{Document, Schema, FAST, STORED, TEXT},
    DocAddress, Index, ReloadPolicy, Score,
};

impl Puzzle {
    fn new() -> Self {
        Self {
            content: MerkleReg::new(),
        }
    }

    fn wrap(&mut self, key: &str, value: &str) {
        self.content.apply(
            self.content
                .write(Pair(key.into(), value.into()), Default::default()),
        );
    }

    fn search(&self, input: &str) -> Result<Vec<String>, Error> {
        let schema = schema();
        let index = Index::try_from(self)?;

        let mut index_writer = index.writer(100_000_000)?;
        dbg!(self);
        for Pair(key, value) in self.content.read().values() {
            let mut document = Document::default();

            if let Some(field) = schema.get_field("key") {
                document.add_bytes(field, key.as_bytes());
            }

            dbg!("hello");
            if let Some(field) = schema.get_field("value") {
                dbg!(&value);
                document.add_text(field, value);
            }

            index_writer.add_document(document);
        }

        index_writer.commit()?;

        let reader = index.reader()?;

        let searcher = reader.searcher();

        let query_parser = QueryParser::for_index(
            &index,
            schema
                .fields()
                .filter(|(_, entry)| entry.is_indexed())
                .map(|entry| entry.0)
                .collect(),
        );

        let query = query_parser.parse_query(input)?;

        let top_docs: Vec<(Score, DocAddress)> =
            searcher.search(&query, &TopDocs::with_limit(10))?;

        let mut output = vec![];

        for (_score, key) in top_docs {
            let found = searcher.doc(key)?;
            output.push(schema.to_json(&found));
        }

        Ok(output)
    }
}

fn schema() -> Schema {
    let mut schema_builder = Schema::builder();

    schema_builder.add_bytes_field("key", FAST | STORED);
    schema_builder.add_text_field("value", TEXT | STORED);

    schema_builder.build()
}

use std::convert::TryFrom;

impl<'a> TryFrom<&'a Puzzle> for Index {
    type Error = Error;

    fn try_from(puzzle: &Puzzle) -> Result<Self, Error> {
        use std::io::ErrorKind;

        let target = painting::frame()?;
        let target = target.join("puzzle");

        match target.metadata() {
            Err(error) => match error.kind() {
                ErrorKind::NotFound => std::fs::create_dir(&target)?,
                _ => {}
            },
            _ => {}
        };

        let target = MmapDirectory::open(target)?;

        let schema = schema();

        Index::open_or_create(target, schema.clone()).map_err(Error::from)
    }
}
