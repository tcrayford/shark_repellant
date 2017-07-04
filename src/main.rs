extern crate git2;
extern crate ansi_term;
use git2::Error;
use git2::Repository;
use git2::RepositoryState;
use git2::Reference;
use git2::Statuses;
use ansi_term::Colour::Red;
use ansi_term::Colour::Green;
use std::string::ToString;

fn main() {
    let _ = match Repository::discover(".") {
        Ok(repo) => run(repo),
        Err(e) => panic!("failed to init: {}", e),
    };
}

fn run(repo: Repository) {
    let state = repo.state();
    match state {
        RepositoryState::Rebase => {
            match repo.revparse_ext("ORIG_HEAD") {
                Ok((_, Some(r))) => {
                    print!("{}{}|REBASE", format_head(Ok(r)), format_statuses(repo.statuses(None)))
                }
                Ok(_) => {
                    print!("{}{}{}", format_head(repo.head()), format_statuses(repo.statuses(None)), format_state(state))
                }
                Err(_) => {
                    print!("{}{}{}", format_head(repo.head()), format_statuses(repo.statuses(None)), format_state(state))
                }
            }
        }
        _ => {
            print!("{}{}{}", format_head(repo.head()), format_statuses(repo.statuses(None)), format_state(state))
        }
    };
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
        Ok(head) => {
            if head.is_branch() {
                let shorthand = head.shorthand();
                match shorthand {
                    Some(name) => String::from(name),
                    None => String::from(""),
                }
            } else {
                match commit_shortid_from_reference(head) {
                    Ok(shortid) => shortid,
                    Err(_) => String::from("HEAD"),
                }
            }
        },
        Err(_) => String::from(""),
    }
}

fn commit_shortid_from_reference(r: Reference) -> Result<String, Error> {
    let peeled = try!(r.peel(git2::ObjectType::Commit));
    let _ = try!(peeled.as_commit().ok_or(git2::Error::from_str("HEAD")));
    let shortid = try!(peeled.short_id());
    let shortid_str = try!(shortid.as_str().ok_or(git2::Error::from_str("HEAD")));
    return Ok(format!("({}...)", shortid_str))
}

fn format_statuses(statuses_result: Result<Statuses, Error>) -> String {
    return match statuses_result {
        Ok(statuses) => dirty_markers(statuses),
        Err(_) => String::from("")
    }
}

fn dirty_markers(statuses: Statuses) -> String {
    let mut changes_in_index = false;
    let mut changes_in_workdir = false;
    let mut added_in_workdir = false;

    for entry in statuses.iter() {
        let s = entry.status();
        if s.contains(git2::STATUS_INDEX_NEW) {
            changes_in_index = true
        }
        if s.contains(git2::STATUS_INDEX_MODIFIED) {
            changes_in_index = true
        }
        if s.contains(git2::STATUS_INDEX_DELETED) {
            changes_in_index = true
        }
        if s.contains(git2::STATUS_INDEX_RENAMED) {
            changes_in_index = true
        }
        if s.contains(git2::STATUS_INDEX_TYPECHANGE) {
            changes_in_index = true
        }

        if s.contains(git2::STATUS_WT_NEW) {
            added_in_workdir = true
        }
        if s.contains(git2::STATUS_WT_MODIFIED) {
            changes_in_workdir = true
        }
        if s.contains(git2::STATUS_WT_DELETED) {
            changes_in_workdir = true
        }
        if s.contains(git2::STATUS_WT_RENAMED) {
            changes_in_workdir = true
        }
        if s.contains(git2::STATUS_WT_TYPECHANGE) {
            changes_in_workdir = true
        }
    }
    let mut result = String::from("");
    if changes_in_workdir {
        result = format!("{}{}", result, Red.paint("!").to_string())
    }
    if changes_in_index {
        result = format!("{}{}", result, Green.paint("+").to_string())
    }
    if added_in_workdir {
        result = format!("{}{}", result, Red.paint("%").to_string())
    }
    return result
}
