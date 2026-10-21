//! Deletes the worktrees the user confirmed, if they still qualify.

use crate::git;
use crate::plan::{Plan, Status, Worktree};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum OutcomeStatus {
    Deleted,
    Skipped,
    Failed,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BranchOutcome {
    pub name: String,
    /// Commit the branch pointed to. `git branch <name> <sha>` restores it
    /// until git garbage-collects the commits.
    pub sha: Option<String>,
    pub deleted: bool,
    pub note: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Outcome {
    pub path: PathBuf,
    pub status: OutcomeStatus,
    pub reasons: Vec<String>,
    pub branch: Option<BranchOutcome>,
}

pub fn apply(plan: &Plan, paths: &[PathBuf], force: bool) -> Vec<Outcome> {
    paths
        .iter()
        .map(
            |path| match plan.worktrees.iter().find(|w| same_path(&w.path, path)) {
                None => outcome(
                    path,
                    OutcomeStatus::Failed,
                    "not a T3 worktree matching the selection",
                ),
                Some(worktree) => apply_one(worktree, force),
            },
        )
        .collect()
}

fn apply_one(worktree: &Worktree, force: bool) -> Outcome {
    match worktree.status {
        Status::Recent => {
            return outcome(
                &worktree.path,
                OutcomeStatus::Skipped,
                "active within the selected period",
            );
        }
        Status::Skipped if !force || !worktree.forceable => {
            return Outcome {
                path: worktree.path.clone(),
                status: OutcomeStatus::Skipped,
                reasons: worktree.reasons.clone(),
                branch: None,
            };
        }
        _ => {}
    }
    let repo = &worktree.project_root;
    let sha = worktree
        .branch
        .as_deref()
        .and_then(|branch| git::branch_sha(repo, branch).ok());
    if let Err(error) = git::remove_worktree(repo, &worktree.path, force) {
        return outcome(&worktree.path, OutcomeStatus::Failed, &error.to_string());
    }
    let branch = worktree.branch.clone().map(|name| {
        let result = if worktree.delete_branch {
            git::delete_branch(repo, &name).map_err(|error| error.to_string())
        } else {
            Err("default branch is kept".to_string())
        };
        BranchOutcome {
            deleted: result.is_ok(),
            note: result.err(),
            name,
            sha,
        }
    });
    Outcome {
        path: worktree.path.clone(),
        status: OutcomeStatus::Deleted,
        reasons: Vec::new(),
        branch,
    }
}

fn outcome(path: &Path, status: OutcomeStatus, reason: &str) -> Outcome {
    Outcome {
        path: path.to_path_buf(),
        status,
        reasons: vec![reason.to_string()],
        branch: None,
    }
}

fn same_path(a: &Path, b: &Path) -> bool {
    let canonical = |path: &Path| fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    canonical(a) == canonical(b)
}
