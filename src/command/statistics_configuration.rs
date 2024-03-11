#![allow(deprecated)]

use crate::git::*;
use anyhow::{anyhow, Context, Result};
use chrono::Duration;
use clap::{arg_enum, value_t};
use git2::BranchType;
use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;

clap::arg_enum! {
    #[derive(PartialEq, Debug)]
    pub enum OutputFormat {
        Stdout,
        Json
    }
}

#[derive(Debug)]
pub struct Configuration {
    pub max_commit_diff: Duration,
    pub first_commit_addition: Duration,
    pub since: CommitTimeBound,
    pub until: CommitTimeBound,
    pub merge_requests: bool,
    pub git_repo_path: PathBuf,
    pub email_aliases: HashMap<String, String>,
    pub branch: Option<String>,
    pub branch_type: BranchType,
    pub output_format: OutputFormat,
}

fn parse_email_alias(s: &str) -> Result<(String, String)> {
    let mut splitter = s.splitn(2, '=');
    match splitter.next() {
        Some(a) => match splitter.next() {
            Some(b) => Ok((a.to_string(), b.to_string())),
            None => Err(anyhow!("Could not parse email alias '{}'", s)),
        },
        None => Err(anyhow!("Could not parse email alias '{}'", s)),
    }
}

pub fn parse_arguments(args: &clap::ArgMatches) -> Result<Configuration> {
    let args_stats = args.subcommand_matches("stats").unwrap();

    let max_commit_diff = args_stats
        .value_of("max-commit-diff")
        .unwrap()
        .parse::<u32>()
        .context("Failed to parse max commit diff to u32.")?;
    let first_commit_addition = args_stats
        .value_of("first-commit-add")
        .unwrap()
        .parse::<u32>()
        .context("Failed to parse first commit add to u32.")?;
    let since = match args_stats.value_of("since") {
        Some(s) => CommitTimeBound::from_str(s)?,
        None => CommitTimeBound::Always,
    };
    let until = match args_stats.value_of("until") {
        Some(s) => CommitTimeBound::from_str(s)?,
        None => CommitTimeBound::Always,
    };
    let merge_requests = args_stats.is_present("merge-requests");
    let git_repo_path = args_stats.value_of("REPO_PATH").unwrap();
    let aliases = match args_stats.values_of("email") {
        Some(vs) => {
            let vec: Vec<&str> = vs.collect();
            let results: Result<Vec<(String, String)>, anyhow::Error> =
                vec.iter().try_fold(Vec::new(), |mut acc, e| {
                    let alias = parse_email_alias(e)?;
                    acc.push(alias);
                    Ok(acc)
                });
            results?
        }
        None => Vec::new(),
    }
    .into_iter()
    .collect::<HashMap<String, String>>();
    let branch = args_stats.value_of("branch").map(|b| b.to_string());
    let branch_type = match args_stats.value_of("branch-type") {
        None => BranchType::Local,
        Some("local") => BranchType::Local,
        Some("remote") => BranchType::Remote,
        Some(x) => return Err(anyhow!("Invalid branch type '{}'", x)),
    };
    let output_format = value_t!(args_stats, "format", OutputFormat).unwrap();

    Ok(Configuration {
        max_commit_diff: Duration::minutes(max_commit_diff.into()),
        first_commit_addition: Duration::minutes(first_commit_addition.into()),
        since,
        until,
        merge_requests,
        git_repo_path: PathBuf::from(git_repo_path),
        email_aliases: aliases,
        branch,
        branch_type,
        output_format,
    })
}
