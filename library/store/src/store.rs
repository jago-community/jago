author::error!(NoHome, std::io::Error, heed::Error);

#[test]
fn test_store() {
    let mut storage = Storage::new(None).unwrap();

    put(
        &mut storage,
        vec![
            ("hello", 1i32.into()),
            ("there", 4.into()),
            ("fellow", 3.into()),
            ("hooman", 2.into()),
            ("oneway", Value::UnsignedInteger8(Vec::from(&b"oneway"[..]))),
        ]
        .into_iter()
        .collect(),
    )
    .unwrap();

    let got = get(&mut storage, ["hello", "hooman", "oneway"]).unwrap();
    let want = vec![
        Some(1.into()),
        Some(2.into()),
        Some(Value::UnsignedInteger8(Vec::from(&b"oneway"[..]))),
    ];

    assert_eq!(got, want);
}

use heed::{BytesDecode, BytesEncode, Database, Env, EnvOpenOptions};
use serde::{Deserialize, Serialize};

pub type Key = str;

use std::borrow::Cow;

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub enum Value {
    UnsignedInteger8(Vec<u8>),
    Integer32(i32),
}

impl<'a> BytesEncode<'a> for Value {
    type EItem = Value;

    fn bytes_encode(
        input: &'a Self::EItem,
    ) -> Result<Cow<'a, [u8]>, Box<dyn std::error::Error + 'static>> {
        let output = bincode::serialize(&input)?;
        Ok(output.into())
    }
}

impl<'a> BytesDecode<'a> for Value {
    type DItem = Value;

    fn bytes_decode(input: &'a [u8]) -> Result<Self::DItem, Box<dyn std::error::Error + 'static>> {
        let output = bincode::deserialize(&input)?;
        Ok(output)
    }
}

impl From<i32> for Value {
    fn from(input: i32) -> Self {
        Self::Integer32(input)
    }
}

pub struct Storage {
    context: Env,
    container: Database<heed::types::Str, Value>,
}

impl Storage {
    pub fn new<'a>(which: Option<&'a str>) -> Result<Self, Error> {
        let target = target_directory()?;

        std::fs::create_dir_all(dbg!(&target))?;

        let context = EnvOpenOptions::new().open(target)?;

        match context.open_database(which) {
            Ok(maybe) => match maybe {
                Some(database) => Ok(Storage {
                    context,
                    container: database,
                }),
                None => context
                    .create_database(which)
                    .map_err(Error::from)
                    .map(|database| Storage {
                        context,
                        container: database,
                    }),
            },
            Err(error) => Err(Error::from(error)),
        }
    }
}

use std::collections::HashMap;

pub fn put<'a>(storage: &mut Storage, input: HashMap<&'a Key, Value>) -> Result<(), Error> {
    let mut writer = storage.context.write_txn()?;
    for (key, value) in input {
        storage.container.put(&mut writer, key, &value)?;
    }
    writer.commit().map_err(Error::from)
}

pub fn get<'a>(
    storage: &'a Storage,
    want: impl AsRef<[&'a Key]>,
) -> Result<Vec<Option<Value>>, Error> {
    let slice = want.as_ref();

    let mut output = Vec::with_capacity(slice.len());

    let reader = storage.context.read_txn()?;

    for key in slice {
        let value = storage.container.get(&reader, key).map(|maybe| {
            maybe.map(|value| match value {
                Value::UnsignedInteger8(value) => Value::UnsignedInteger8(value),
                Value::Integer32(value) => Value::Integer32(value),
            })
        })?;
        output.push(value);
    }

    Ok(output)
}

use std::path::PathBuf;

fn target_directory() -> Result<PathBuf, Error> {
    dirs::home_dir().map_or(Err(Error::NoHome), |path| {
        Ok(path
            .join("target")
            .join("jago")
            .join("library")
            .join("store"))
    })
}

#[cfg(test)]
pub fn storage_directory() -> Result<PathBuf, Error> {
    target_directory()
}
