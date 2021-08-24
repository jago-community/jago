author::error!(NoHome, std::io::Error, heed::Error);

#[test]
fn test_store() {
    let store = store().unwrap();

    let mut storage = storage(&store, None).unwrap();

    put(
        &store,
        &mut storage,
        vec![("hello", 1i32), ("there", 4), ("fellow", 3), ("hooman", 2)]
            .into_iter()
            .collect(),
    )
    .unwrap();

    let got = get(&store, &mut storage, ["hello", "hooman"]).unwrap();
    let want = vec![Some(1), Some(2)];

    assert_eq!(got, want);
}

use std::{fs, path::Path};

use heed::{types::OwnedType, Database, Env, EnvOpenOptions};

type Store = Env;

type Key = str;
type Value = i32;

type Storage = Database<heed::types::Str, OwnedType<Value>>;

fn store() -> Result<Store, Error> {
    let target = target_directory()?;

    std::fs::create_dir_all(&target)?;

    EnvOpenOptions::new().open(target).map_err(Error::from)
}

fn storage<'a>(store: &'a Store, which: Option<&'a str>) -> Result<Storage, Error> {
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
    storage: &mut Storage,
    want: impl AsRef<[&'a Key]>,
) -> Result<Vec<Option<Value>>, Error> {
    let slice = want.as_ref();

    let mut output = Vec::with_capacity(slice.len());

    let mut reader = store.read_txn()?;

    for key in slice {
        let value = storage.get(&mut reader, key)?;
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
