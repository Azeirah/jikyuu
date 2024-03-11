#![allow(deprecated)]

extern crate anyhow;
extern crate chrono;
extern crate clap;
extern crate prettytable;
extern crate regex;
extern crate serde;
extern crate serde_json;

use anyhow::{anyhow, Result};
use chrono::{Duration, Local, TimeZone, Utc};
use git2::{BranchType, Commit, Repository};
use regex::Regex;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::string::ToString;

use crate::command::statistics_print::print_results;
use crate::git::CommitHours;
use crate::ExitCode;

use super::statistics_configuration::{parse_arguments, Configuration};

/// Get commits of a specific repository branch.
fn get_commits<'repo>(
    repo: &'repo Repository,
    branch: &Option<String>,
    branch_kind: BranchType,
) -> Result<Vec<Commit<'repo>>> {
    let refs = repo.references()?;

    let ref_prefix = match branch_kind {
        BranchType::Local => "heads",
        BranchType::Remote => "remotes",
    };

    let branch_refs = match branch {
        Some(b) => {
            let s = format!("refs/{}/{}", ref_prefix, b);
            let mut vec = Vec::new();
            for r in refs {
                let r = r?;
                let name = r.name();
                if let Some(name) = name {
                    if name == s {
                        vec.push(r)
                    }
                }
            }
            vec
        }
        None => {
            let mut vec = Vec::new();
            let rx = Regex::new(&format!("refs/{}/.*", ref_prefix))?;
            for r in refs {
                let r = r?;
                let name = r.name();
                if let Some(name) = name {
                    if rx.is_match(name) {
                        vec.push(r)
                    }
                }
            }
            vec
        }
    };

    let mut result = Vec::new();
    let mut seen = HashSet::new();
    for r in branch_refs.iter() {
        if let Some(latest_oid) = r.target() {
            let mut revwalk = repo.revwalk()?;
            revwalk.set_sorting(git2::Sort::TIME | git2::Sort::REVERSE)?;
            revwalk.push(latest_oid)?;
            for oid in revwalk {
                let oid = oid?;
                if !seen.contains(&oid) {
                    let commit = repo.find_commit(oid)?;
                    result.push(commit.clone());
                    seen.insert(oid);
                }
            }
        }
    }

    Ok(result)
}

// Filter out commits in a given time period.
fn filter_commits<'repo>(
    configuration: &Configuration,
    commits: Vec<Commit<'repo>>,
) -> Vec<Commit<'repo>> {
    let since = configuration.since.to_date_time();
    let until = configuration.until.to_date_time();

    let since_local = since.map(|b| Local.from_local_datetime(&b).unwrap());
    let until_local = until.map(|b| Local.from_local_datetime(&b).unwrap());

    commits
        .into_iter()
        .filter(|commit| {
            let time = commit.time();
            if let Some(bound) = since_local {
                let dt = Utc.timestamp(time.seconds(), 0);
                if dt < bound {
                    return false;
                }
            }
            if let Some(bound) = until_local {
                let dt = Utc.timestamp(time.seconds(), 0);
                if dt > bound {
                    return false;
                }
            }

            configuration.merge_requests
                || !commit
                    .summary()
                    .map(|s| s.starts_with("Merge "))
                    .unwrap_or(false)
        })
        .collect()
}

// Collect time estimate by author.
fn estimate_author_time(
    mut commits: Vec<&Commit>,
    email: Option<String>,
    max_commit_diff: &Duration,
    first_commit_addition: &Duration,
) -> CommitHours {
    let author_name = commits[0].author().name().map(|n| n.to_string());

    commits.sort_by_key(|c| c.time());

    let len = commits.len() - 1;
    let all_but_last = commits.iter().enumerate().take(len);
    let duration = all_but_last.fold(Duration::minutes(0), |acc, (i, commit)| {
        let next_commit = commits.get(i + 1).unwrap();
        let diff_seconds = next_commit.time().seconds() - commit.time().seconds();
        let dur = Duration::seconds(diff_seconds);

        if dur < *max_commit_diff {
            acc + dur
        } else {
            acc + *first_commit_addition
        }
    });

    CommitHours {
        email,
        author_name,
        duration,
        commit_count: commits.len(),
    }
}

/// Collect time estimates by author.
fn estimate_author_times(configuration: &Configuration, commits: Vec<Commit>) -> Vec<CommitHours> {
    let mut no_email: Vec<&Commit> = Vec::new();
    let mut by_email: HashMap<String, Vec<&Commit>> = HashMap::new();
    for commit in &commits {
        let author = commit.author();
        let email = author
            .email()
            .map(|e| match configuration.email_aliases.get(e) {
                Some(alias) => alias,
                None => e,
            });

        let author_commits = match email {
            Some(e) => by_email.entry(e.to_string()).or_default(),
            None => &mut no_email,
        };

        author_commits.push(commit);
    }

    let mut result = Vec::new();
    if !no_email.is_empty() {
        result.push(estimate_author_time(
            no_email,
            None,
            &configuration.max_commit_diff,
            &configuration.first_commit_addition,
        ));
    }
    for (email, author_commits) in by_email {
        result.push(estimate_author_time(
            author_commits,
            Some(email),
            &configuration.max_commit_diff,
            &configuration.first_commit_addition,
        ));
    }
    result.sort_by(|a, b| {
        let ord = b.duration.cmp(&a.duration);
        if ord != Ordering::Equal {
            return ord;
        }
        b.commit_count.cmp(&a.commit_count)
    });

    result
}

/// Get the git repository context - whether.
pub fn get_git_context(path_directory: PathBuf) -> Result<Repository> {
    let repository = match Repository::discover(&path_directory) {
        Ok(repo) => repo,
        Err(e) => return Err(anyhow!("Failed to open repository: {}", e)),
    };
    let submodules = repository.submodules()?;

    log::debug!(
        "Repository: {:?} Submodules: {:?}",
        repository.path(),
        submodules.len()
    );

    for submodule in submodules {
        let submodule_path = submodule.path();
        log::debug!(
            "Submodule Path: {:?} Path: {:?}",
            submodule_path,
            path_directory
        );
        if submodule_path == path_directory {
            return Ok(submodule.open()?);
        }
    }

    Ok(repository)
}

/// Run statistics on repository.
pub fn statistics(args: &clap::ArgMatches) -> Result<ExitCode> {
    let configuration = &parse_arguments(args)?;
    log::debug!("{:?}", configuration);
    log::debug!("");

    let repository = get_git_context(configuration.git_repo_path.clone())?;
    log::debug!("Repository: {:?}", repository.path());
    log::debug!("");

    let commits = get_commits(
        &repository,
        &configuration.branch,
        configuration.branch_type,
    )?;
    log::debug!("Commits: {:?}", commits);
    log::debug!("");

    let commits_filtered = filter_commits(configuration, commits);
    log::debug!("Commits Filtered: {:?}", commits_filtered);
    log::debug!("");

    let estimate_by_author = estimate_author_times(configuration, commits_filtered);
    log::debug!("Estimate: {:?}", estimate_by_author);
    log::debug!("");

    if estimate_by_author.is_empty() {
        match &configuration.branch {
            Some(b) => {
                let branch_type = match configuration.branch_type {
                    BranchType::Local => "local",
                    BranchType::Remote => "remote",
                };
                return Err(anyhow!(
                    "No commits found for branch '{}' ({}).",
                    b,
                    branch_type
                ));
            }
            None => {
                return Err(anyhow!("No commits found.",));
            }
        }
    } else {
        print_results(&estimate_by_author, &configuration.output_format)?;
    };

    log::debug!("Done.");
    log::debug!("");

    Ok(0)
}
