mod format;
mod identity;
mod serve;
mod write;

use bytes::Bytes;
use either::Either;

use crate::document;
use crate::input::Input;

type Context = Option<hyper::Request<hyper::Body>>;

type Output = Bytes;

//pub enum Output {
//Bytes(Bytes),
//File(library::file_system::File),
//}

pub async fn handle<'a>(input: &'a Input<'a>) -> Result<Option<Output>, Error> {
    match input {
        Input::Serve(_) => {
            serve::handle().await?;
        }
        input @ _ => return handle_core(&input),
    };

    Ok(None)
}

pub fn handle_core<'a>(input: &'a Input<'a>) -> Result<Option<Output>, Error> {
    let mut output = None;

    match input {
        Input::Check(Some(inner)) => {
            match inner.as_ref() {
                &Input::Rest(ref maybe_reference) => match check(Either::Right(maybe_reference)) {
                    Err(error) => {
                        eprintln!("check {} failed: {}", maybe_reference, error);
                    }
                    _ => {
                        println!("all good");
                    }
                },
                _ => {
                    eprintln!("unexpected pattern following check: {:?}", inner);
                }
            };
        }
        Input::Check(None) => {
            output = check(Either::Left("/jago"))?;
        }
        Input::Serve(_) => {
            return Err(Error::NestedServe);
        }
        Input::Prepare => {
            let root = dirs::home_dir().unwrap().join("local/jago");
            std::fs::copy(root.join("jago"), root.join("README.md"))?;
        }
        Input::Rest(ref maybe_path) => {
            output = check(Either::Left(maybe_path)).or_else(|_| {
                check(Either::Right(
                    maybe_path.strip_prefix("/").unwrap_or(maybe_path),
                ))
            })?;
        }
    };

    Ok(output)
}

#[test]
#[ignore]
fn test_handle() {
    {
        // avoid pass phrase checking for keys
        std::fs::create_dir_all(dirs::home_dir().unwrap().join("cache/jago")).unwrap();
        std::fs::copy(
            dirs::home_dir().unwrap().join("local/jago/jago"),
            dirs::home_dir().unwrap().join("cache/jago/jago"),
        )
        .unwrap();
    }

    let got = tokio_test::block_on(handle(&Input::Check(None))).unwrap();
    let want = include_str!("../jago");

    assert_eq!(got, Some(bytes::Bytes::from(want)));
}

fn check(maybe_address: Either<&str, &str>) -> Result<Option<Output>, Error> {
    let home = dirs::home_dir().unwrap();

    let cache = home.join("cache");

    if !cache.exists() {
        match std::fs::create_dir_all(&cache) {
            Err(error) => {
                eprintln!("unexpected error while opening repository: {}", error);
                std::process::exit(1);
            }
            _ => {}
        };
    }

    let path = match maybe_address {
        Either::Right(rest) => {
            let address = crate::address::parse(rest)?;
            let source = address.source();
            let identity = std::env::var("IDENTITY")
                .or_else(
                    |_: std::env::VarError| -> Result<String, Box<dyn std::error::Error>> {
                        Ok(String::from(".ssh/id_rsa"))
                    },
                )
                .map(|identity| home.join(identity))
                .unwrap();

            let path = cache.join(address.source());

            ensure_repository(&path, identity, source)?;

            path.join(address.path().unwrap_or(address.name()))
        }
        Either::Left(rest) => home
            .join("local/jago")
            .join(rest.strip_prefix("/").unwrap_or(rest)),
    };

    //if crate::image::is_supported(&path) {
    //return file_stream(&path, context).map(|file| Some(Output::File(filer));
    if path.exists() {
        return content(&path).map(|bytes| Some(bytes));
    }

    Err(Error::WeirdPath(path, WhyWeird::NotThere))
}

fn file_stream(
    path: &std::path::Path,
    context: Context,
) -> Result<library::file_system::File, Error> {
    let request = match context {
        Some(request) => request,
        _ => return Err(Error::BadContext),
    };

    let conditionals = library::file_system::conditionals(&request)?;

    unimplemented!()
}

fn ensure_repository<'a>(
    path: &'a std::path::Path,
    identity: std::path::PathBuf,
    source: &'a str,
) -> Result<(), Error> {
    let mut callbacks = git2::RemoteCallbacks::new();

    let mut public_key = identity.clone();
    public_key.set_extension("pub");

    callbacks.credentials(move |_url, username_from_url, _allowed_types| {
        git2::Cred::ssh_key(
            username_from_url.unwrap(),
            Some(&public_key),
            &identity,
            None,
        )
    });

    match git2::Repository::open(&path) {
        Ok(repository) => {
            println!("coin flip");
            if rand::random() {
                println!("not checking");
                return Ok((/*trust*/));
            }
            let remote_name = "origin";
            let remote_branch = "main";
            let mut remote = repository.find_remote(remote_name)?;
            let commit = fetch_remote(&repository, &[remote_branch], &mut remote, callbacks)?;
            merge_remote(&repository, "main", commit)?;
        }
        Err(error) => {
            match error.code() {
                git2::ErrorCode::NotFound => {
                    let mut fo = git2::FetchOptions::new();
                    fo.remote_callbacks(callbacks);

                    let mut builder = git2::build::RepoBuilder::new();
                    builder.fetch_options(fo);

                    match builder.clone(source, path) {
                        Err(error) => {
                            println!("{:?} -> {:?}", source, path);
                            eprintln!("unexpected error while cloning repository: {}", error);
                            std::process::exit(1);
                        }
                        _ => {}
                    };
                }
                _ => {
                    eprintln!("unexpected error while opening repository: {}", error);
                    std::process::exit(1);
                }
            };
        }
    };

    Ok(())
}

// BEGIN git@github.com:rust-lang/git2-rs@master/examples/pull.rs

/*
 * libgit2 "pull" example - shows how to pull remote data into a local branch.
 *
 * Written by the libgit2 contributors
 *
 * To the extent possible under law, the author(s) have dedicated all copyright
 * and related and neighboring rights to this software to the public domain
 * worldwide. This software is distributed without any warranty.
 *
 * You should have received a copy of the CC0 Public Domain Dedication along
 * with this software. If not, see
 * <http://creativecommons.org/publicdomain/zero/1.0/>.
 */

fn fetch_remote<'a>(
    repo: &'a git2::Repository,
    refs: &[&str],
    remote: &'a mut git2::Remote,
    mut handlers: git2::RemoteCallbacks,
) -> Result<git2::AnnotatedCommit<'a>, Error> {
    use std::io::Write;

    // Print out our transfer progress.
    handlers.transfer_progress(|stats| {
        if stats.received_objects() == stats.total_objects() {
            print!(
                "Resolving deltas {}/{}\r",
                stats.indexed_deltas(),
                stats.total_deltas()
            );
        } else if stats.total_objects() > 0 {
            print!(
                "Received {}/{} objects ({}) in {} bytes\r",
                stats.received_objects(),
                stats.total_objects(),
                stats.indexed_objects(),
                stats.received_bytes()
            );
        }
        std::io::stdout().flush().unwrap();
        true
    });

    let mut fo = git2::FetchOptions::new();
    fo.remote_callbacks(handlers);
    // Always fetch all tags.
    // Perform a download and also update tips
    fo.download_tags(git2::AutotagOption::All);
    println!("Fetching {} for repo", remote.name().unwrap());
    remote.fetch(refs, Some(&mut fo), None)?;

    // If there are local objects (we got a thin pack), then tell the user
    // how many objects we saved from having to cross the network.
    let stats = remote.stats();
    if stats.local_objects() > 0 {
        println!(
            "\rReceived {}/{} objects in {} bytes (used {} local \
             objects)",
            stats.indexed_objects(),
            stats.total_objects(),
            stats.received_bytes(),
            stats.local_objects()
        );
    } else {
        println!(
            "\rReceived {}/{} objects in {} bytes",
            stats.indexed_objects(),
            stats.total_objects(),
            stats.received_bytes()
        );
    }

    let fetch_head = repo.find_reference("FETCH_HEAD")?;
    Ok(repo.reference_to_annotated_commit(&fetch_head)?)
}

fn fast_forward(
    repo: &git2::Repository,
    lb: &mut git2::Reference,
    rc: &git2::AnnotatedCommit,
) -> Result<(), git2::Error> {
    let name = match lb.name() {
        Some(s) => s.to_string(),
        None => String::from_utf8_lossy(lb.name_bytes()).to_string(),
    };
    let msg = format!("Fast-Forward: Setting {} to id: {}", name, rc.id());
    println!("{}", msg);
    lb.set_target(rc.id(), &msg)?;
    repo.set_head(&name)?;
    repo.checkout_head(Some(
        git2::build::CheckoutBuilder::default()
            // For some reason the force is required to make the working directory actually get updated
            // I suspect we should be adding some logic to handle dirty working directory states
            // but this is just an example so maybe not.
            .force(),
    ))?;
    Ok(())
}

fn normal_merge(
    repo: &git2::Repository,
    local: &git2::AnnotatedCommit,
    remote: &git2::AnnotatedCommit,
) -> Result<(), git2::Error> {
    let local_tree = repo.find_commit(local.id())?.tree()?;
    let remote_tree = repo.find_commit(remote.id())?.tree()?;
    let ancestor = repo
        .find_commit(repo.merge_base(local.id(), remote.id())?)?
        .tree()?;
    let mut idx = repo.merge_trees(&ancestor, &local_tree, &remote_tree, None)?;

    if idx.has_conflicts() {
        println!("Merge conficts detected...");
        repo.checkout_index(Some(&mut idx), None)?;
        return Ok(());
    }
    let result_tree = repo.find_tree(idx.write_tree_to(repo)?)?;
    // now create the merge commit
    let msg = format!("Merge: {} into {}", remote.id(), local.id());
    let sig = repo.signature()?;
    let local_commit = repo.find_commit(local.id())?;
    let remote_commit = repo.find_commit(remote.id())?;
    // Do our merge commit and set current branch head to that commit.
    let _merge_commit = repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        &msg,
        &result_tree,
        &[&local_commit, &remote_commit],
    )?;
    // Set working tree to match head.
    repo.checkout_head(None)?;
    Ok(())
}

fn merge_remote<'a>(
    repo: &'a git2::Repository,
    remote_branch: &str,
    fetch_commit: git2::AnnotatedCommit<'a>,
) -> Result<(), Error> {
    // 1. do a merge analysis
    let analysis = repo.merge_analysis(&[&fetch_commit])?;

    // 2. Do the appropriate merge
    if analysis.0.is_fast_forward() {
        println!("Doing a fast forward");
        // do a fast forward
        let refname = format!("refs/heads/{}", remote_branch);
        match repo.find_reference(&refname) {
            Ok(mut r) => {
                fast_forward(repo, &mut r, &fetch_commit)?;
            }
            Err(_) => {
                // The branch doesn't exist so just set the reference to the
                // commit directly. Usually this is because you are pulling
                // into an empty repository.
                repo.reference(
                    &refname,
                    fetch_commit.id(),
                    true,
                    &format!("Setting {} to {}", remote_branch, fetch_commit.id()),
                )?;
                repo.set_head(&refname)?;
                repo.checkout_head(Some(
                    git2::build::CheckoutBuilder::default()
                        .allow_conflicts(true)
                        .conflict_style_merge(true)
                        .force(),
                ))?;
            }
        };
    } else if analysis.0.is_normal() {
        // do a normal merge
        let head_commit = repo.reference_to_annotated_commit(&repo.head()?)?;
        normal_merge(&repo, &head_commit, &fetch_commit)?;
    } else {
        println!("Nothing to do...");
    }
    Ok(())
}

// END git@github.com:rust-lang/git2-rs@master/examples/pull.rs

fn content(path: &std::path::Path) -> Result<Bytes, Error> {
    let metadata = std::fs::metadata(path)?;

    let file = match metadata.is_file() {
        true => std::fs::File::open(path)?,
        false => {
            let name = match path.file_name() {
                Some(name) => name,
                None => return Err(Error::WeirdPath(path.into(), WhyWeird::NoName)),
            };

            let path = path.join(name).to_path_buf();

            let metadata = std::fs::metadata(&path)
                .map_err(|error| Error::WeirdPath(path.clone(), WhyWeird::Machine(error)))?;

            match metadata.is_file() {
                true => std::fs::File::open(path)?,
                false => {
                    return Err(Error::WeirdPath(
                        path.to_path_buf(),
                        WhyWeird::RepeatedDirectoryName,
                    ))
                }
            }
        }
    };

    let reader = std::io::BufReader::new(file);

    let output = vec![];
    let mut writer = std::io::BufWriter::new(output);

    document::html(
        reader,
        &mut writer,
        path.file_stem().and_then(|stem| stem.to_str()),
    )?;

    let buffer = writer.into_inner()?;

    Ok(Bytes::from(buffer))
}

#[derive(Debug)]
pub enum Error {
    Machine(std::io::Error),
    Repository(git2::Error),
    Serve(serve::Error),
    Address(crate::address::Error),
    Write(write::Error),
    Encoding(std::string::FromUtf8Error),
    Identity(identity::Error),
    Document(document::Error),
    WeirdPath(std::path::PathBuf, WhyWeird),
    Post(std::io::IntoInnerError<std::io::BufWriter<Vec<u8>>>),
    FileSystem(library::file_system::Error),
    BadContext,
    NestedServe,
}

#[derive(Debug)]
pub enum WhyWeird {
    // Technically, fine. Just don't want to deal with walking yet.
    RepeatedDirectoryName,
    NoName,
    NotThere,
    Machine(std::io::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NestedServe => write!(f, "can't nest serve requests"),
            Error::BadContext => write!(f, "tried an operation without needed context"),
            Error::Machine(error) => write!(f, "{}", error),
            Error::Repository(error) => write!(f, "{}", error),
            Error::Serve(error) => write!(f, "{}", error),
            Error::Address(error) => write!(f, "{}", error),
            Error::Encoding(error) => write!(f, "{}", error),
            Error::Write(error) => write!(f, "{}", error),
            Error::Identity(error) => write!(f, "{}", error),
            Error::Document(error) => write!(f, "{}", error),
            Error::Post(error) => write!(f, "{}", error),
            Error::FileSystem(error) => write!(f, "{}", error),
            Error::WeirdPath(path, why) => {
                write!(f, "weird path: {}\n\n", path.display())?;

                match why {
                    WhyWeird::RepeatedDirectoryName => write!(
                        f,
                        "nesting directory names is fine, I just haven't felt the need to walk yet"
                    ),
                    WhyWeird::NoName => write!(f, "if a file has no name, does it really exist?"),
                    WhyWeird::NotThere => write!(f, "not found"),
                    WhyWeird::Machine(error) => write!(f, "rage againsT {}", error),
                }
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Machine(error) => Some(error),
            Error::Repository(error) => Some(error),
            Error::Serve(error) => Some(error),
            Error::Address(error) => Some(error),
            Error::Encoding(error) => Some(error),
            Error::Write(error) => Some(error),
            Error::Identity(error) => Some(error),
            Error::Document(error) => Some(error),
            Error::Post(error) => Some(error),
            Error::FileSystem(error) => Some(error),
            Error::WeirdPath(_, why) => match why {
                WhyWeird::Machine(error) => Some(error),
                _ => None,
            },
            Error::NestedServe => None,
            Error::BadContext => None,
        }
    }
}

impl From<identity::Error> for Error {
    fn from(error: identity::Error) -> Self {
        Self::Identity(error)
    }
}

impl From<document::Error> for Error {
    fn from(error: document::Error) -> Self {
        Self::Document(error)
    }
}

impl From<library::file_system::Error> for Error {
    fn from(error: library::file_system::Error) -> Self {
        Self::FileSystem(error)
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Machine(error)
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(error: std::string::FromUtf8Error) -> Self {
        Self::Encoding(error)
    }
}

impl From<std::io::IntoInnerError<std::io::BufWriter<Vec<u8>>>> for Error {
    fn from(error: std::io::IntoInnerError<std::io::BufWriter<Vec<u8>>>) -> Self {
        Self::Post(error)
    }
}

impl From<git2::Error> for Error {
    fn from(error: git2::Error) -> Self {
        Self::Repository(error)
    }
}

impl From<serve::Error> for Error {
    fn from(error: serve::Error) -> Self {
        Self::Serve(error)
    }
}

impl From<write::Error> for Error {
    fn from(error: write::Error) -> Self {
        Self::Write(error)
    }
}

impl From<crate::address::Error> for Error {
    fn from(error: crate::address::Error) -> Self {
        Self::Address(error)
    }
}
