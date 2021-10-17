#[cfg(feature = "scrape")]
mod scrape;

mod web;

#[cfg(feature = "search")]
#[test]
fn fill_scratch() {
    let context = b"fill_scratch".to_vec();

    let mut scratch = Scratch::carve();

    let data = vec![
        (
            b"The Old Man and the Sea".to_vec(),
            "He was an old man who fished alone in a skiff in the Gulf Stream and \
             he had gone eighty-four days now without taking a fish.",
        ),
        (
            b"Of Mice and Men".to_vec(),
            "A few miles south of Soledad, the Salinas River drops in close to the hillside \
            bank and runs deep and green. The water is warm too, for it has slipped twinkling \
            over the yellow sands in the sunlight before reaching the narrow pool. On one \
            side of the river the golden foothill slopes curve up to the strong and rocky \
            Gabilan Mountains, but on the valley side the water is lined with trees—willows \
            fresh and green with every spring, carrying in their lower leaf junctures the \
            debris of the winter’s flooding; and sycamores with mottled, white, recumbent \
            limbs and branches that arch over the pool",
        ),
        (
            b"Frankenstein".to_vec(),
            "You will rejoice to hear that no disaster has accompanied the commencement of an \
             enterprise which you have regarded with such evil forebodings.  I arrived here \
             yesterday, and my first task is to assure my dear sister of my welfare and \
             increasing confidence in the success of my undertaking.",
        ),
    ];

    for (key, value) in data.iter() {
        scratch.fill(&context, key, value).unwrap();
    }

    let aspects = scratch.aspects().collect::<Vec<(Key, Value)>>();

    let want = data
        .clone()
        .into_iter()
        .map(|(key, value)| (key.into(), bincode::serialize(value).unwrap()))
        .collect::<Vec<(Key, Value)>>();

    for (key, value) in want {
        assert!(scratch.covers(&key, Some(value)).unwrap());
    }

    let matches = search(&scratch, "vi").unwrap();

    assert_eq!(
        data.clone()
            .into_iter()
            .skip(2)
            .map(|(key, value)| (key.into(), bincode::serialize(value).unwrap()))
            .collect::<Vec<(Key, Value)>>(),
        matches
    );
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Bincode {0}")]
    Bincode(#[from] bincode::Error),
    #[cfg(feature = "search")]
    #[error("Painting {0}")]
    Painting(#[from] painting::Error),
    #[cfg(feature = "search")]
    #[error("Index {0}")]
    Index(#[from] tantivy::TantivyError),
    #[cfg(feature = "search")]
    #[error("Query {0}")]
    Query(#[from] tantivy::query::QueryParserError),
    #[cfg(feature = "search")]
    #[error("Machine {0}")]
    Machine(#[from] std::io::Error),
    #[cfg(feature = "search")]
    #[error("IndexTarget {0}")]
    IndexTarget(#[from] tantivy::directory::error::OpenDirectoryError),
    #[cfg(feature = "search")]
    #[error("NoKey {0}")]
    NoKey(String),
    #[cfg(feature = "search")]
    #[error("NoValue {0}")]
    NoValue(String),
    #[cfg(feature = "search")]
    #[error("UnexpectedType {0}")]
    UnexpectedType(String, tantivy::schema::Value),
}

use crdts::{merkle_reg::MerkleReg, CmRDT, MVReg, Map};

pub type Context = Vec<u8>;
pub type Key = Vec<u8>;
type Value = Vec<u8>;
type Perspective = MVReg<Value, Context>;

pub struct Scratch {
    void: Map<Key, Perspective, Context>,
    expanse: Map<Key, MVReg<MerkleReg<Value>, Context>, Context>,
    //expanse: Map<Key, MVReg<MerkleReg<Value>, Context>, Context>,
}

use serde::Serialize;

impl Scratch {
    fn carve() -> Self {
        Self {
            void: Default::default(),
            expanse: Default::default(),
        }
    }

    fn fill(&mut self, context: &Context, key: &Key, value: impl Serialize) -> Result<(), Error> {
        let value = bincode::serialize(&value)?;

        let read = self.void.is_empty();

        let operation = self.void.update(
            key.clone(),
            read.derive_add_ctx(context.clone()),
            |register, ctx| {
                // ...
                register.write(value, ctx)
            },
        );

        self.void.apply(operation);

        Ok(())
    }

    fn observe(
        &mut self,
        context: &Context,
        key: &Key,
        value: impl Serialize,
    ) -> Result<(), Error> {
        let value = bincode::serialize(&value)?;

        let read = self.expanse.is_empty();

        let operation = self.expanse.update(
            key.clone(),
            read.derive_add_ctx(context.clone()),
            |register, context| {
                let mut next = MerkleReg::default();

                for previous in register.read().val {
                    next.apply(next.write(value.clone(), Default::default()));
                }

                register.write(next, context)
            },
        );

        self.expanse.apply(operation);

        Ok(())
    }

    fn aspects(&self) -> impl Iterator<Item = (Key, Value)> + '_ {
        self.void
            .iter()
            .map(|item_ctx| (item_ctx.val.0.clone(), item_ctx.val.1.read().val[0].clone()))
    }

    fn covers(&self, key: &Key, value: Option<Value>) -> Result<bool, Error> {
        self.void
            .get(key)
            .val
            .map(|register| register.read().val)
            .map_or(Ok(false), |values| match value {
                Some(value) => Ok(values.contains(&value)),
                None => Ok(values.len() > 0),
            })
    }
}

#[cfg(feature = "search")]
use tantivy::{
    collector::TopDocs,
    directory::MmapDirectory,
    query::{FuzzyTermQuery, QueryParser},
    schema::{self, Document, Schema, FAST, STORED, TEXT},
    DocAddress, Index, ReloadPolicy, Score, Term,
};

#[cfg(feature = "search")]
fn search(scratch: &Scratch, pattern: &str) -> Result<Vec<(Key, Value)>, Error> {
    let schema = schema();
    let index = index(scratch)?;

    let reader = index.reader()?;

    let searcher = reader.searcher();

    let key_field = schema
        .get_field("key")
        .map_or_else(|| Err(Error::NoKey("key".into())), Ok)?;
    let value_field = schema
        .get_field("value")
        .map_or_else(|| Err(Error::NoKey("value".into())), Ok)?;

    /*let query_parser = QueryParser::for_index(
        &index,
        schema
            .fields()
            .filter(|(_, entry)| entry.is_indexed())
            .map(|entry| entry.0)
            .collect(),
    );

    let query = query_parser.parse_query(pattern)?;
    */

    let term = Term::from_field_text(value_field, pattern);
    let query = FuzzyTermQuery::new(term, 1, true);

    let top_docs: Vec<(Score, DocAddress)> = searcher.search(&query, &TopDocs::with_limit(10))?;

    let mut output = vec![];

    for (score, key) in top_docs {
        let found = searcher.doc(key)?;

        let key = found
            .get_first(key_field)
            .map_or_else(|| Err(Error::NoValue("key".into())), Ok)?;
        let value = found
            .get_first(value_field)
            .map_or_else(|| Err(Error::NoValue("value".into())), Ok)?;

        let key = match key {
            schema::Value::Bytes(bytes) => bytes,
            _ => return Err(Error::UnexpectedType("key".into(), key.clone())),
        };
        let value = match value {
            schema::Value::Str(string) => string,
            _ => return Err(Error::UnexpectedType("value".into(), value.clone())),
        };

        output.push((key.clone(), bincode::serialize(value)?));
    }

    Ok(output)
}

#[cfg(feature = "search")]
fn index(scratch: &Scratch) -> Result<Index, Error> {
    use std::{collections::HashSet, io::ErrorKind};

    let target = painting::frame()?;
    let target = target.join("scratch");

    match target.metadata() {
        Err(error) => match error.kind() {
            ErrorKind::NotFound => std::fs::create_dir(&target)?,
            _ => {}
        },
        _ => {}
    };

    let target = MmapDirectory::open(target)?;

    let schema = schema();

    let index = Index::open_or_create(target, schema.clone())?;

    let meta = index.load_metas()?;

    let mut documents = HashSet::new();

    if meta.segments.len() > 0 {
        let key = schema
            .get_field("key")
            .map_or_else(|| Err(Error::NoKey("key".into())), Ok)?;

        let reader = index.reader()?;
        let searcher = reader.searcher();

        for segment in searcher.segment_readers() {
            let keys = segment.fast_fields().bytes(key)?;

            for document in segment.doc_ids_alive() {
                let key = keys.get_bytes(document);
                documents.insert(key.to_vec());
            }
        }
    }

    let mut index_writer = index.writer(100_000_000)?;

    for (key, mut value) in scratch.aspects() {
        if documents.contains(&key) {
            continue;
        }

        let mut document = Document::default();

        if let Some(field) = schema.get_field("key") {
            document.add_bytes(field, key);
        }

        if let Some(field) = schema.get_field("value") {
            let value = bincode::deserialize::<&str>(&value)?;
            document.add_text(field, &value);
        }

        index_writer.add_document(document);
    }

    index_writer.commit()?;

    Ok(index)
}

#[cfg(feature = "search")]
fn schema() -> Schema {
    let mut schema_builder = Schema::builder();

    schema_builder.add_bytes_field("key", FAST | STORED);
    schema_builder.add_text_field("value", TEXT | STORED);

    schema_builder.build()
}
