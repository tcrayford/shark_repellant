extern crate git2;
extern crate ansi_term;
use git2::Error;
use git2::Repository;
use git2::RepositoryState;
use git2::Reference;
use git2::Statuses;
use ansi_term::Colour::{Red, Green};
use ansi_term::ANSIStrings;
use std::string::ToString;
use std::path::Path;
use std::fs::File;
use std::io;
use std::io::Read;
use std::fmt;

fn main() {
    let pwd_info = match pwd() {
        Ok(result) => result,
        Err(e) => format!("{}", e),
    };
    match git() {
        Ok(result) => print!("\n{}({}) $ ", pwd_info, result),
        Err(_) => print!("\n{} $ ", pwd_info),

    }
}

fn pwd() -> Result<String, Error> {
    match std::env::current_dir() {
        Ok(maybe_dir) => match maybe_dir.file_name() {
            Some(dir) => match dir.to_str() {
                Some(dirname) => Ok(dirname.to_string()),
                None => Err(git2::Error::from_str("")),
            },
            None => Err(git2::Error::from_str("")),
        },
        Err(e) => Err(git2::Error::from_str(&format!("{}", e)))
    }
}

fn git() -> Result<String, Error> {
    match Repository::discover(".") {
        Ok(repo) => git_info(&repo),
        Err(e) => Err(e)
    }
}

fn git_info(repo: &Repository) -> Result<String, Error> {
    let state = repo.state();
    let statuses = format_statuses(repo.statuses(None));
    let head = format_head(repo.head());
    match state {
        RepositoryState::Rebase | RepositoryState::RebaseInteractive => format_rebase(rebase_info(repo), &statuses[..], &head[..], state),
        _ => {
            Ok(format!("{}{}{}", head, statuses, format_state(state)))
        }
    }
}

fn format_rebase(info: Result<RebaseInfo, Error>, statuses: &str, head: &str, state: RepositoryState) -> Result<String, Error> {
    match info {
        Ok(r) => {
            match r.branch {
                Some(branch) => Ok(format!("{}{}|{} {}/{}", branch, statuses, r.rebase_type, r.step, r.total)),
                None => Ok(format!("{}|{} {}/{}", statuses, r.rebase_type, r.step, r.total)),

            }
        }
        Err(e) => Ok(format!("{}{}{}{}", e, head, statuses, format_state(state))),
    }
}

fn format_state(state: RepositoryState) -> String {
    let state_string = match state {
        RepositoryState::Clean => "",
        RepositoryState::Merge => "MERGE",
        RepositoryState::Revert | RepositoryState::RevertSequence => "REVERT",
        RepositoryState::CherryPick | RepositoryState::CherryPickSequence => "CHERRY-PICK",
        RepositoryState::Bisect => "BISECT",
        RepositoryState::Rebase | RepositoryState::RebaseInteractive | RepositoryState::RebaseMerge => "REBASE",
        RepositoryState::ApplyMailbox | RepositoryState::ApplyMailboxOrRebase => "MAILBOX",
    };
    String::from(state_string)
}

fn format_head(head_result: Result<Reference, Error>) -> String {
    match head_result {
        Ok(head) => {
            if head.is_branch() {
                let shorthand = head.shorthand();
                match shorthand {
                    Some(name) => String::from(name),
                    None => String::from(""),
                }
            } else {
                match commit_shortid_from_reference(&head) {
                    Ok(shortid) => shortid,
                    Err(_) => String::from("HEAD"),
                }
            }
        },
        Err(_) => String::from(""),
    }
}

fn commit_shortid_from_reference(r: &Reference) -> Result<String, Error> {
    let peeled = try!(r.peel(git2::ObjectType::Commit));
    let _ = try!(peeled.as_commit().ok_or_else(|| git2::Error::from_str("HEAD")));
    let shortid = try!(peeled.short_id());
    let shortid_str = try!(shortid.as_str().ok_or_else(|| git2::Error::from_str("HEAD")));
    Ok(format!("({}...)", shortid_str))
}

fn format_statuses(statuses_result: Result<Statuses, Error>) -> String {
    match statuses_result {
        Ok(statuses) => dirty_markers(&statuses),
        Err(_) => String::from("")
    }
}

fn dirty_markers(statuses: &Statuses) -> String {
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
    let mut strings = vec![];
    if changes_in_workdir {
        strings.push(Red.paint("!"));
    }
    if changes_in_index {
        strings.push(Green.paint("+"));
    }
    if added_in_workdir {
        strings.push(Red.paint("%"));
    }
    format!("{}", ANSIStrings(&strings[..]))
}

#[derive(Copy,Clone)]
enum RebaseType {
    Plain,
    Interactive,
    Merge,
    ApplyMerge,
    ApplyMergeRebase,
}

impl fmt::Display for RebaseType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let printable = match *self {
            RebaseType::Plain => "REBASE",
            RebaseType::Interactive => "REBASE-i",
            RebaseType::Merge => "REBASE-m",
            RebaseType::ApplyMerge => "AM",
            RebaseType::ApplyMergeRebase => "AM/REBASE",
        };
        write!(f, "{}", printable)
    }
}

struct RebaseInfo {
    branch: Option<String>,
    step: String,
    total: String,
    rebase_type: RebaseType,
}

fn rebase_info(repo: &Repository) -> Result<RebaseInfo, Error> {
    if repo.path().join("rebase-merge").exists() {
        let head_ref_name = try!(read_file_git_error(repo.path().join("rebase-merge").join("head-name").as_path()));
        let branch = format_head(repo.find_reference(&head_ref_name));
        let step = try!(read_file_git_error(repo.path().join("rebase-merge").join("msgnum").as_path()));
        let total = try!(read_file_git_error(repo.path().join("rebase-merge").join("end").as_path()));
        if repo.path().join("rebase-merge").join("interactive").exists() {
            Ok(RebaseInfo {branch: Some(branch), step: step, total: total, rebase_type: RebaseType::Interactive})
        } else {
            Ok(RebaseInfo {branch: Some(branch), step: step, total: total, rebase_type: RebaseType::Merge})
        }
    } else if repo.path().join("rebase-apply").exists() {
        let head_ref_name = try!(read_file_git_error(repo.path().join("rebase-apply").join("head-name").as_path()));
        let branch = format_head(repo.find_reference(&head_ref_name));
        let step = try!(read_file_git_error(repo.path().join("rebase-apply").join("next").as_path()));
        let total = try!(read_file_git_error(repo.path().join("rebase-apply").join("last").as_path()));
        if repo.path().join("rebase-apply").join("rebasing").exists() {
            Ok(RebaseInfo {branch: Some(branch), step: step, total: total, rebase_type: RebaseType::Plain})
        } else if repo.path().join("rebase-apply").join("applying").exists() {
            Ok(RebaseInfo {branch: Some(branch), step: step, total: total, rebase_type: RebaseType::ApplyMerge})
        } else {
            Ok(RebaseInfo {branch: Some(branch), step: step, total: total, rebase_type: RebaseType::ApplyMergeRebase})
        }
    } else {
        Err(git2::Error::from_str("HEAD"))
    }
}

fn read_file_git_error(p: &Path) -> Result<String, git2::Error> {
    match read_file(p) {
        Ok(contents) => Ok(contents),
        Err(e) => Err(git2::Error::from_str(&format!("{}", e)))
    }
}

fn read_file(p: &Path) -> Result<String, io::Error> {
    let mut file = try!(File::open(p));
    let mut contents = String::new();
    let _ = file.read_to_string(&mut contents);
    Ok(String::from(contents.trim()))
}
