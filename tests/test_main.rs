extern crate assert_cmd;
extern crate predicates;
extern crate pretty_assertions;
extern crate tempfile;

use assert_cmd::prelude::*;
use chrono::DateTime;
use git2::{Oid, Repository, Signature, Time};
use predicates::prelude::*;
use std::process::Command;
use tempfile::tempdir;

const BIN: &str = "jikyuu";

fn create_commit_initial(
    repository: &Repository,
    time: String,
) -> Result<Oid, Box<dyn std::error::Error>> {
    let username = "Nate-Wilkins";
    let email = "nate-wilkins@code-null.com";

    let signature = Signature::new(
        username,
        email,
        &Time::new(DateTime::parse_from_rfc2822(&time).unwrap().timestamp(), 0),
    )?;

    let oid = repository.index().unwrap().write_tree().unwrap();
    let tree = repository.find_tree(oid).unwrap();
    let oid_commit = repository
        .commit(Some("HEAD"), &signature, &signature, "init 1", &tree, &[])
        .unwrap();

    Ok(oid_commit)
}

/// Create a commit in the provided repository at a specific time.
fn create_commit(
    repository: &Repository,
    time: String,
    message: String,
) -> Result<Oid, Box<dyn std::error::Error>> {
    let username = "Nate-Wilkins";
    let email = "nate-wilkins@code-null.com";

    let mut index = repository.index()?;
    index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)?;

    let tree_id = index.write_tree()?;
    let tree = repository.find_tree(tree_id)?;
    let parent_commit = repository.head().unwrap().peel_to_commit().unwrap();

    let signature = Signature::new(
        username,
        email,
        &Time::new(DateTime::parse_from_rfc2822(&time).unwrap().timestamp(), 0),
    )?;
    let oid_commit = repository.commit(
        Some("HEAD"),
        &signature,
        &signature,
        &message,
        &tree,
        &[&parent_commit],
    )?;

    Ok(oid_commit)
}

#[test]
fn test_command_completions_type_zsh() -> Result<(), Box<dyn std::error::Error>> {
    // Given the CLI.
    let mut cmd = Command::cargo_bin(BIN)?;

    // When the user generates completions for zsh.
    let result = cmd.arg("completions").arg("--type").arg("zsh").assert();

    result
        // Then no errors occurred.
        .success()
        .stderr(predicate::str::is_empty())
        // Then completions for zsh were outputted.
        .stdout(predicate::str::contains(format!("#compdef {}", BIN)));

    Ok(())
}

#[test]
fn test_command_statistics_repository() -> Result<(), Box<dyn std::error::Error>> {
    // Given the CLI.
    let mut cmd = Command::cargo_bin(BIN)?;

    // And we have a repository.
    let path_repository = tempdir().unwrap().path().join("");
    let repository = Repository::init(&path_repository)?;
    create_commit_initial(&repository, String::from("Wed, 18 Feb 2015 10:10:09 GMT"))?;
    create_commit(
        &repository,
        String::from("Wed, 18 Feb 2015 11:10:09 GMT"),
        String::from("Commit A"),
    )?;
    create_commit(
        &repository,
        String::from("Wed, 18 Feb 2015 12:01:00 GMT"),
        String::from("Commit B"),
    )?;

    // When the user runs the command statistics.
    let result = cmd.arg("statistics").arg(path_repository).assert();

    result
        // Then no errors occurred.
        .success()
        .stderr(predicate::str::is_empty())
        // Then statistics were outputed for the repository.
        .stdout(predicate::str::contains(
            "
+--------------+----------------------------+---------+-----------------+
| Author       | Email                      | Commits | Estimated Hours |
|              |                            |         |                 |
| Nate-Wilkins | nate-wilkins@code-null.com | 3       | 1.8333334       |
|              |                            |         |                 |
| Total        |                            | 3       | 1.8333334       |
+--------------+----------------------------+---------+-----------------+
"
            .trim(),
        ));

    Ok(())
}

#[test]
fn test_command_statistics_submodule() -> Result<(), Box<dyn std::error::Error>> {
    // Given the CLI.
    let mut cmd = Command::cargo_bin(BIN)?;

    // And we have a repository submodule.
    let path_repository_a = tempdir().unwrap().path().join("");
    let repository_a = Repository::init(&path_repository_a)?;
    create_commit_initial(&repository_a, String::from("Wed, 18 Feb 2015 10:10:09 GMT"))?;
    create_commit(
        &repository_a,
        String::from("Wed, 18 Feb 2015 12:01:00 GMT"),
        String::from("Commit A A"),
    )?;
    create_commit(
        &repository_a,
        String::from("Wed, 18 Feb 2015 03:11:09 GMT"),
        String::from("Commit A B"),
    )?;

    // And we have a repository that has the submodule.
    let path_repository_main = tempdir().unwrap().path().join("");
    let repository_main = Repository::init(&path_repository_main)?;
    let path_repository_main_submodule_a = path_repository_main.join("packages/submodule_a");
    let repository_a_url = format!(
        "file://{}.git",
        path_repository_a.into_os_string().to_str().unwrap()
    );
    let mut submodule = repository_main.submodule(
        &repository_a_url,
        path_repository_main_submodule_a
            .strip_prefix(path_repository_main)
            .unwrap(),
        true,
    )?;
    submodule.open()?;
    submodule.clone(None)?;

    create_commit_initial(
        &repository_main,
        String::from("Wed, 18 Feb 2015 10:10:09 GMT"),
    )?;
    create_commit(
        &repository_main,
        String::from("Wed, 18 Feb 2015 15:10:09 GMT"),
        String::from("Commit Main Submodule"),
    )?;

    println!("{:?}", submodule.open()?.references().unwrap().count());

    // When the user runs the command statistics.
    let result = cmd
        .arg("--verbosity")
        .arg("debug")
        .arg("statistics")
        .arg(path_repository_main_submodule_a)
        .assert();

    result
        // Then no errors occurred.
        .success()
        .stderr(predicate::str::is_empty())
        // Then statistics were outputed for the repository.
        .stdout(predicate::str::contains(
            "
+--------------+----------------------------+---------+-----------------+
| Author       | Email                      | Commits | Estimated Hours |
|              |                            |         |                 |
| Nate-Wilkins | nate-wilkins@code-null.com | 3       | 2.3333333       |
|              |                            |         |                 |
| Total        |                            | 3       | 2.3333333       |
+--------------+----------------------------+---------+-----------------+
"
            .trim(),
        ));

    Ok(())
}
