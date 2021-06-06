use crate::address::Address;

#[test]
fn test_ensure() {
    fn clear<'a>(address: &'a Address) -> Result<(), Error> {
        let context = dirs::home_dir().unwrap();
        let path = address.directory(context.join("cache"));
        let metadata = std::fs::metadata(&path)?;

        if metadata.is_file() {
            std::fs::remove_file(&path).map_err(Error::from)
        } else {
            std::fs::remove_dir_all(&path).map_err(Error::from)
        }
    }

    use crate::address;

    let address = address::parse("git@github.com:jago-community/jago.git/favicon.ico").unwrap();

    clear(&address).unwrap();
    ensure(&address).unwrap();

    let mut target = std::io::BufWriter::new(vec![]);
    let path = std::sync::Arc::new(address.full(dirs::home_dir().unwrap().join("cache")));
    let _ = crate::source::read(&mut target, path).unwrap();
    let buffer = target.into_inner().unwrap();

    let reader = image::io::Reader::new(std::io::Cursor::new(buffer))
        .with_guessed_format()
        .unwrap();

    let format = reader.format().unwrap();

    assert_eq!(format, image::ImageFormat::Ico);

    reader.decode().unwrap();
}

pub fn ensure<'a>(address: &'a Address) -> Result<(), Error> {
    let context = dirs::home_dir().unwrap();

    let identity = context.join("local/jago/keys/id_rsa");

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

    let path = address.directory(context.join("cache"));

    let source = address.source();

    match git2::Repository::open(&path) {
        Ok(repository) => {
            println!("coin flip");
            if rand::random() {
                println!("not checking");
                return Ok(());
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

                    println!("cloning: {} -> {}", source, path.display());

                    match builder.clone(source.as_ref(), &path) {
                        Err(error) => {
                            eprintln!("Unexpected error while cloning repository: {}", error);
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

#[derive(Debug)]
pub enum Error {
    Machine(std::io::Error),
    Repository(git2::Error),
    Address(crate::address::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Machine(error) => write!(f, "{}", error),
            Error::Repository(error) => write!(f, "{}", error),
            Error::Address(error) => write!(f, "{}", error),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Machine(error) => Some(error),
            Error::Repository(error) => Some(error),
            Error::Address(error) => Some(error),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Machine(error)
    }
}

impl From<git2::Error> for Error {
    fn from(error: git2::Error) -> Self {
        Self::Repository(error)
    }
}

impl From<crate::address::Error> for Error {
    fn from(error: crate::address::Error) -> Self {
        Self::Address(error)
    }
}
