//! Groups threads by worktree and decides which worktrees may be deleted.

use crate::git;
use crate::store::ThreadRow;
use chrono::{DateTime, TimeDelta, Utc};
use serde::Serialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

pub struct Selection {
    pub older_than: TimeDelta,
    pub projects: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    /// Inactive and safe to delete.
    Candidate,
    /// Inactive, but deleting it would lose work.
    Skipped,
    /// Active within the selected period.
    Recent,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Thread {
    pub id: String,
    pub title: String,
    pub settled_at: Option<String>,
    pub last_activity_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Worktree {
    pub path: PathBuf,
    pub project: String,
    pub project_root: PathBuf,
    pub status: Status,
    pub reasons: Vec<String>,
    /// False when the safety checks failed, so `--force` can't override the skip.
    #[serde(skip)]
    pub forceable: bool,
    pub last_activity_at: DateTime<Utc>,
    pub size_bytes: u64,
    /// Branch checked out in the worktree; `None` for a detached HEAD.
    pub branch: Option<String>,
    /// Whether deleting the worktree also deletes `branch`.
    pub delete_branch: bool,
    pub threads: Vec<Thread>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Plan {
    pub cutoff: DateTime<Utc>,
    pub candidates: usize,
    pub candidate_bytes: u64,
    pub skipped: usize,
    pub recent: usize,
    /// Candidates and skipped worktrees, least recently active first.
    pub worktrees: Vec<Worktree>,
}

impl Plan {
    pub fn build(threads: Vec<ThreadRow>, selection: &Selection, now: DateTime<Utc>) -> Plan {
        let cutoff = now - selection.older_than;
        let mut groups: BTreeMap<PathBuf, Vec<ThreadRow>> = BTreeMap::new();
        for thread in threads {
            groups
                .entry(thread.worktree_path.clone())
                .or_default()
                .push(thread);
        }
        // Filter whole worktrees, so threads of other projects still count as activity.
        if !selection.projects.is_empty() {
            groups.retain(|_, threads| {
                threads
                    .iter()
                    .any(|thread| selection.projects.iter().any(|p| matches(p, thread)))
            });
        }
        let mut worktrees: Vec<Worktree> = groups
            .into_iter()
            // Paths T3 or the user already removed stay in the database.
            .filter(|(path, _)| path.is_dir())
            .map(|(path, threads)| evaluate(path, threads, cutoff))
            .collect();
        worktrees.sort_by_key(|worktree| worktree.last_activity_at);
        let count = |status| worktrees.iter().filter(|w| w.status == status).count();
        let (candidates, skipped, recent) = (
            count(Status::Candidate),
            count(Status::Skipped),
            count(Status::Recent),
        );
        let candidate_bytes = worktrees
            .iter()
            .filter(|worktree| worktree.status == Status::Candidate)
            .map(|worktree| worktree.size_bytes)
            .sum();
        Plan {
            cutoff,
            candidates,
            candidate_bytes,
            skipped,
            recent,
            worktrees,
        }
    }

    /// The plan as printed by `plan`: recent worktrees are only counted.
    pub fn without_recent(mut self) -> Plan {
        self.worktrees
            .retain(|worktree| worktree.status != Status::Recent);
        self
    }
}

fn matches(project: &str, thread: &ThreadRow) -> bool {
    let root = fs::canonicalize(project).unwrap_or_else(|_| PathBuf::from(project));
    project == thread.project_title || project == thread.project_id || root == thread.project_root
}

fn evaluate(path: PathBuf, threads: Vec<ThreadRow>, cutoff: DateTime<Utc>) -> Worktree {
    let last_activity_at = threads
        .iter()
        .map(|thread| thread.activity_at)
        .max()
        .unwrap_or_default();
    let first = &threads[0];
    let mut worktree = Worktree {
        project: first.project_title.clone(),
        project_root: first.project_root.clone(),
        status: Status::Recent,
        reasons: Vec::new(),
        forceable: false,
        last_activity_at,
        size_bytes: 0,
        branch: None,
        delete_branch: false,
        threads: threads
            .iter()
            .map(|thread| Thread {
                id: thread.thread_id.clone(),
                title: thread.title.clone(),
                settled_at: thread.settled_at.clone(),
                last_activity_at: thread.activity_at,
            })
            .collect(),
        path,
    };
    if last_activity_at > cutoff {
        return worktree;
    }
    worktree.size_bytes = dir_size(&worktree.path);
    (worktree.reasons, worktree.forceable) = match blocking_reasons(&worktree.path) {
        Ok(reasons) => (reasons, true),
        Err(error) => (vec![error.to_string()], false),
    };
    worktree.status = if worktree.reasons.is_empty() {
        Status::Candidate
    } else {
        Status::Skipped
    };
    worktree.branch = git::current_branch(&worktree.path).ok().flatten();
    worktree.delete_branch = worktree
        .branch
        .as_deref()
        .is_some_and(|branch| !is_protected(&worktree.project_root, branch));
    worktree
}

/// Reasons that deleting the worktree and its branch would lose work.
fn blocking_reasons(path: &Path) -> anyhow::Result<Vec<String>> {
    let mut reasons = Vec::new();
    if git::has_local_changes(path)? {
        reasons.push("uncommitted or untracked changes".to_string());
    }
    let unpushed = git::unpushed_commits(path)?;
    if unpushed > 0 {
        reasons.push(format!("{unpushed} commits not on any remote"));
    }
    Ok(reasons)
}

fn is_protected(repo: &Path, branch: &str) -> bool {
    ["main", "master"].contains(&branch) || git::default_branch(repo).as_deref() == Some(branch)
}

fn dir_size(path: &Path) -> u64 {
    let Ok(entries) = fs::read_dir(path) else {
        return 0;
    };
    entries
        .flatten()
        .map(|entry| match entry.file_type() {
            Ok(kind) if kind.is_dir() => dir_size(&entry.path()),
            _ => entry.metadata().map(|m| m.len()).unwrap_or(0),
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn activity_exactly_at_the_cutoff_is_inactive() {
        let dir = tempfile::TempDir::new().unwrap();
        let now = Utc::now();
        let thread = ThreadRow {
            thread_id: "t1".into(),
            title: "t1".into(),
            project_id: "p1".into(),
            project_title: "demo".into(),
            project_root: dir.path().into(),
            worktree_path: dir.path().into(),
            settled_at: None,
            activity_at: now - TimeDelta::days(4),
        };
        let selection = Selection {
            older_than: TimeDelta::days(4),
            projects: Vec::new(),
        };

        let plan = Plan::build(vec![thread], &selection, now);

        assert_eq!(plan.recent, 0);
    }
}
