extern crate git2;
use git2::Error;
use git2::Repository;
use git2::RepositoryState;
use git2::Reference;

fn main() {
    let _ = match Repository::discover(".") {
        Ok(repo) => run(repo),
        Err(e) => panic!("failed to init: {}", e),
    };
}

fn run(repo: Repository) {
    // TODO:
    // print dirty markers/etc
    print!("{}{}", format_head(repo.head()), format_state(repo.state()))
}

fn format_state(state: RepositoryState) -> String {
    let state_string = match state {
        RepositoryState::Clean => "",
        RepositoryState::Merge => "MERGE",
        RepositoryState::Revert => "REVERT",
        RepositoryState::RevertSequence => "REVERT",
        RepositoryState::CherryPick => "CHERRY-PICK",
        RepositoryState::CherryPickSequence => "CHERRY-PICK",
        RepositoryState::Bisect => "BISECT",
        RepositoryState::Rebase => "REBASE",
        RepositoryState::RebaseInteractive => "REBASE",
        RepositoryState::RebaseMerge => "REBASE",
        RepositoryState::ApplyMailbox => "MAILBOX",
        RepositoryState::ApplyMailboxOrRebase => "MAILBOX",
    };
    return String::from(state_string)
}

fn format_head(head_result: Result<Reference, Error>) -> String {
    return match head_result {
        Ok(head) => match head.shorthand() {
                        Some(name) => String::from(name),
                        None => String::from(""),
                    },
        Err(_) => String::from(""),
    }
}
