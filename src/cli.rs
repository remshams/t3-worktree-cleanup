//! Command-line arguments.

use chrono::TimeDelta;
use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

/// Delete T3 Code worktrees whose threads have been inactive for a while.
///
/// Run `plan` to preview, then `apply` with the paths to delete. Both print JSON.
/// Worktrees with uncommitted changes or unpushed commits are skipped unless
/// `apply --force` is given. Deleting a worktree also deletes its local branch,
/// except the default branch.
#[derive(Debug, Parser)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// List inactive worktrees and whether they can be deleted.
    Plan(SelectionArgs),
    /// Delete the given worktrees if they still qualify.
    Apply {
        #[command(flatten)]
        selection: SelectionArgs,
        /// Also delete worktrees with uncommitted changes or unpushed commits.
        #[arg(long)]
        force: bool,
        /// Worktree paths from `plan`.
        #[arg(required = true)]
        paths: Vec<PathBuf>,
    },
}

#[derive(Debug, Args)]
pub struct SelectionArgs {
    /// Minimum inactivity, as days or hours: 4d, 36h.
    #[arg(long, default_value = "4d", value_parser = parse_age)]
    pub older_than: TimeDelta,
    /// Only this project, by title, id or root path. Repeatable.
    #[arg(long = "project")]
    pub projects: Vec<String>,
    /// T3 home directory. Defaults to $T3CODE_HOME, then ~/.t3.
    #[arg(long)]
    pub base_dir: Option<PathBuf>,
}

fn parse_age(value: &str) -> Result<TimeDelta, String> {
    let invalid = || format!("expected a number followed by d or h, got {value:?}");
    let (number, unit): (&str, fn(i64) -> TimeDelta) = if let Some(n) = value.strip_suffix('d') {
        (n, TimeDelta::days)
    } else if let Some(n) = value.strip_suffix('h') {
        (n, TimeDelta::hours)
    } else {
        return Err(invalid());
    };
    Ok(unit(i64::from(
        number.parse::<u32>().map_err(|_| invalid())?,
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_days_and_hours() {
        assert_eq!(parse_age("4d"), Ok(TimeDelta::days(4)));
        assert_eq!(parse_age("36h"), Ok(TimeDelta::hours(36)));
    }

    #[test]
    fn rejects_other_formats() {
        for value in ["", "4", "d", "4w", "-d", "-4d", "1.5d", "4ü"] {
            assert!(parse_age(value).is_err(), "{value:?} should be rejected");
        }
    }
}
