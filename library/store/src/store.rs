author::error!(NoHome, std::io::Error, heed::Error);

#[test]
fn test_store() {
    let store = store().unwrap();

    let mut storage = storage(&store, None).unwrap();

    put(
        &store,
        &mut storage,
        vec![
            ("hello", 1i32.into()),
            ("there", 4.into()),
            ("fellow", 3.into()),
            ("hooman", 2.into()),
            (
                "oneway",
                Value::UnsignedInteger8(Cow::Owned(Vec::from(&b"oneway"[..]))),
            ),
        ]
        .into_iter()
        .collect(),
    )
    .unwrap();

    let got = get(&store, &mut storage, ["hello", "hooman", "oneway"]).unwrap();
    let want = vec![
        Some(1.into()),
        Some(2.into()),
        Some(Value::UnsignedInteger8(Cow::Owned(Vec::from(
            &b"oneway"[..],
        )))),
    ];

    assert_eq!(got, want);
}

use heed::{types::OwnedType, BytesDecode, BytesEncode, Database, Env, EnvOpenOptions};
use serde::{Deserialize, Serialize};

pub type Store = Env;
pub type Key = str;

use std::borrow::Cow;

#[derive(Debug, PartialEq, Deserialize, Serialize)]
enum Value<'a> {
    UnsignedInteger8(Cow<'a, [u8]>),
    Integer32(i32),
}

impl<'a> BytesEncode<'a> for Value<'a> {
    type EItem = Value<'a>;

    fn bytes_encode(
        input: &'a Self::EItem,
    ) -> Result<Cow<'a, [u8]>, Box<dyn std::error::Error + 'static>> {
        let output = bincode::serialize(&input)?;
        Ok(output.into())
    }
}

impl<'a> BytesDecode<'a> for Value<'a> {
    type DItem = Value<'a>;

    fn bytes_decode(input: &'a [u8]) -> Result<Self::DItem, Box<dyn std::error::Error + 'static>> {
        let output = bincode::deserialize(&input)?;
        Ok(output)
    }
}

impl From<i32> for Value<'_> {
    fn from(input: i32) -> Self {
        Self::Integer32(input)
    }
}

type Storage<'a> = Database<heed::types::Str, Value<'a>>;

fn store() -> Result<Store, Error> {
    let target = target_directory()?;

    std::fs::create_dir_all(&target)?;

    EnvOpenOptions::new().open(target).map_err(Error::from)
}

fn storage<'a>(store: &'a Store, which: Option<&'a str>) -> Result<Storage<'a>, Error> {
    match store.open_database(which) {
        Ok(maybe) => match maybe {
            Some(database) => Ok(database),
            None => store.create_database(which).map_err(Error::from),
        },
        Err(error) => Err(Error::from(error)),
    }
}

use std::collections::HashMap;

fn put<'a>(
    store: &'a Store,
    storage: &mut Storage,
    input: HashMap<&'a Key, Value>,
) -> Result<(), Error> {
    let mut writer = store.write_txn()?;
    for (key, value) in input {
        storage.put(&mut writer, key, &value)?;
    }
    writer.commit().map_err(Error::from)
}

fn get<'a>(
    store: &'a Store,
    storage: &mut Storage<'a>,
    want: impl AsRef<[&'a Key]>,
) -> Result<Vec<Option<Value<'a>>>, Error> {
    let slice = want.as_ref();

    let mut output = Vec::with_capacity(slice.len());

    let reader = store.read_txn()?;

    for key in slice {
        let value = storage.get(&reader, key).map(|maybe| {
            maybe.map(|value| match value {
                Value::UnsignedInteger8(value) => {
                    Value::UnsignedInteger8(Cow::Owned(value.into_owned()))
                }
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
