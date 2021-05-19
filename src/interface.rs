mod format;
mod identity;
mod serve;
mod write;

use bytes::Bytes;

use crate::input::Input;

pub async fn handle<'a>(input: &'a Input<'a>) -> Result<Option<Bytes>, Error> {
    match input {
        Input::Serve(_) => {
            serve::handle().await?;
        }
        input @ _ => return handle_core(&input),
    };

    Ok(None)
}

pub fn handle_core<'a>(input: &'a Input<'a>) -> Result<Option<Bytes>, Error> {
    let mut output = None;

    match input {
        Input::Check(Some(inner)) => {
            match inner.as_ref() {
                &Input::Rest(ref maybe_reference) => match check(maybe_reference) {
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
            output = check("git@github.com:jago-community/jago.git")?;
        }
        Input::Serve(_) => {
            return Err(Error::NestedServe);
        }
        Input::Rest(ref maybe_path) => {
            output = check(maybe_path).or_else(|_| {
                check(&format!(
                    "git@github.com:jago-community/jago.git{}",
                    maybe_path
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

fn check(maybe_address: &str) -> Result<Option<bytes::Bytes>, Error> {
    let address = crate::address::parse(maybe_address)?;

    let home = dirs::home_dir().unwrap();

    let identity = std::env::var("IDENTITY")
        .or_else(
            |_: std::env::VarError| -> Result<String, Box<dyn std::error::Error>> {
                Ok(String::from(".ssh/id_rsa"))
            },
        )
        .map(|identity| home.join(identity))
        .unwrap();

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

    let path = cache.join(address.source());

    ensure_repository(&path, identity, address.source())?;

    let location = match address.path() {
        Some(rest) => path.join(&rest),
        None => std::path::PathBuf::from(address.name()),
    };

    if location.exists() {
        return content(&location).map(|bytes| Some(bytes));
    }

    // if empty path, check for file with same name as repository

    Ok(None)
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

    // 2. Do the appopriate merge
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

fn content(path: &std::path::Path) -> Result<bytes::Bytes, Error> {
    use std::io::Read;

    let metadata = std::fs::metadata(path)?;

    // check is_dir

    let mut buffer = vec![];

    let file = std::fs::File::open(path)?;
    let mut reader = std::io::BufReader::new(file);

    reader.read_to_end(&mut buffer)?;

    let buffer = String::from_utf8(buffer)?;

    let output = vec![];
    let mut writer = std::io::BufWriter::new(output);

    write::html(
        &mut writer,
        path.file_stem().and_then(|stem| stem.to_str()),
        &buffer,
    )?;

    Ok(Bytes::from(writer.buffer().to_owned()))
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
    NestedServe,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NestedServe => write!(f, "can't nest serve requests"),
            Error::Machine(error) => write!(f, "{}", error),
            Error::Repository(error) => write!(f, "{}", error),
            Error::Serve(error) => write!(f, "{}", error),
            Error::Address(error) => write!(f, "{}", error),
            Error::Encoding(error) => write!(f, "{}", error),
            Error::Write(error) => write!(f, "{}", error),
            Error::Identity(error) => write!(f, "{}", error),
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
            Error::NestedServe => None,
        }
    }
}

impl From<identity::Error> for Error {
    fn from(error: identity::Error) -> Self {
        Self::Identity(error)
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
