#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Io {0}")]
    Io(#[from] std::io::Error),
    #[error("Repository {0}")]
    Repository(#[from] git2::Error),
    #[error("Environment {0}")]
    Environment(#[from] environment::Error),
}

use ::{
    git2::{
        build::RepoBuilder, Cred, Error as GitError, ErrorClass, ErrorCode, FetchOptions,
        RemoteCallbacks, Repository,
    },
    instrument::prelude::*,
    serde::Deserialize,
    std::{
        collections::hash_map::DefaultHasher,
        hash::{Hash, Hasher},
        path::{Path, PathBuf},
    },
};

use super::DependencyDefinition;

#[derive(Deserialize)]
pub struct Dependency {
    key: String,
    version: Option<String>,
}

impl From<DependencyDefinition> for Dependency {
    fn from(other: DependencyDefinition) -> Self {
        match other {
            DependencyDefinition::Key(key) => Self { key, version: None },
            DependencyDefinition::Definition { key, version } => Self {
                key,
                version: Some(version),
            },
        }
    }
}

impl Dependency {
    pub fn ensure(&self, target: &Path) -> Result<(), Error> {
        info!(
            "ensuring: {}{} {:x}",
            self.key,
            if let Some(version) = &self.version {
                format!("@{}", version)
            } else {
                "".into()
            },
            self.get_target_hash()
        );

        let target = self.get_target(target);

        match Repository::open(&target) {
            Err(error) if error.code() == ErrorCode::NotFound => {
                self.ssh_clone_without_password(&target)
            }
            _ => Ok(()),
        }
    }

    fn get_target(&self, target: &Path) -> PathBuf {
        target
            .to_path_buf()
            .join(format!("{:x}", self.get_target_hash()))
    }

    fn get_target_hash(&self) -> u64 {
        let mut s = DefaultHasher::new();
        self.key.hash(&mut s);
        s.finish()
    }

    fn ssh_clone_without_password(&self, target: &Path) -> Result<(), Error> {
        let id = environment::identity(None)?;

        let mut callbacks = RemoteCallbacks::new();

        callbacks.credentials(|_url, username_from_url, _allowed_types| {
            let user = username_from_url.ok_or(GitError::new(
                ErrorCode::GenericError,
                ErrorClass::None,
                "no username from url",
            ))?;

            Cred::ssh_key(user, None, &id, None)
        });

        let mut fo = FetchOptions::new();

        fo.remote_callbacks(callbacks);

        let mut builder = RepoBuilder::new();

        builder.fetch_options(fo);

        let remote = format!("git@github.com:{}.git", self.key);

        let repository = builder.clone(&remote, target)?;

        if let Some(version) = &self.version {
            repository.set_head(&version)?;

            repository.checkout_head(Some(
                git2::build::CheckoutBuilder::default()
                    // For some reason the force is required to make the working directory actually get updated
                    // I suspect we should be adding some logic to handle dirty working directory states
                    // but this is just an example so maybe not.
                    .force(),
            ))?;
        }

        Ok(())
    }
}
