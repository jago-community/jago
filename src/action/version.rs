#[derive(Debug, PartialEq)]
pub enum Action {
    Write,
}

pub fn parse<I: Iterator<Item = String>>(input: &mut I) -> Result<Option<Action>, Error> {
    let mut action = None;
    while let Some(item) = input.next() {
        action = match &item[..] {
            "write" => Some(Action::Write),
            _ => None,
        };
    }
    Ok(action)
}

pub fn handle(action: Option<Action>) -> Result<(), Error> {
    let action = match action {
        Some(action) => action,
        None => {
            println!("print version help");
            return Ok(());
        }
    };

    let repository = git2::Repository::discover(".")?;

    match action {
        Action::Write => write(&repository)?,
    };

    Ok(())
}

fn write(repository: &git2::Repository) -> Result<(), Error> {
    use uuid::Uuid;

    // Commit to write_{current-branch-name}

    // read current branch name
    let head = match repository.head() {
        Ok(head) => Some(head),
        Err(ref error)
            if error.code() == git2::ErrorCode::UnbornBranch
                || error.code() == git2::ErrorCode::NotFound =>
        {
            None
        }
        Err(error) => return Err(Error::Repository(error)),
    };

    let branch = head.as_ref().and_then(|h| h.shorthand());

    let target = [Some("write"), branch]
        .iter()
        .filter_map(|reference| *reference)
        .collect::<Vec<&str>>()
        .join("_");

    let author = git2::Signature::now("Jago Contributor", "contributors@jago.cafe")?;
    let committer = &author;

    let uuid = Uuid::new_v4();
    let mut buffer = Uuid::encode_buffer();
    let message: &str = uuid.to_simple().encode_lower(&mut buffer);

    let parents: &[&git2::Commit<'_>] = &[/*&Commit<'_>*/];

    let mut index = repository.index()?;
    index.add_all(["."].iter(), git2::IndexAddOption::DEFAULT, None)?;
    let oid = index.write_tree()?;
    let tree = repository.find_tree(oid)?;

    repository.commit(Some(&target), &author, committer, message, &tree, parents)?;

    println!("Target branch: {:?}", target);

    unimplemented!()
}

#[derive(Debug)]
pub enum Error {
    Repository(git2::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Repository(error) => write!(f, "{}", error),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Repository(error) => Some(error),
        }
    }
}

impl From<git2::Error> for Error {
    fn from(error: git2::Error) -> Self {
        Self::Repository(error)
    }
}
