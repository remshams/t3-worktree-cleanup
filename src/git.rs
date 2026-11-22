//! Thin wrappers around the git commands the tool runs.

use anyhow::{Result, bail};
use std::path::Path;
use std::process::Command;

fn git(dir: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git").arg("-C").arg(dir).args(args).output()?;
    if !output.status.success() {
        bail!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// The branch checked out in `worktree`, or `None` for a detached HEAD.
pub fn current_branch(worktree: &Path) -> Result<Option<String>> {
    let branch = git(worktree, &["branch", "--show-current"])?;
    Ok(Some(branch).filter(|name| !name.is_empty()))
}

/// Number of commits on HEAD that no remote-tracking branch contains.
pub fn unpushed_commits(worktree: &Path) -> Result<usize> {
    Ok(git(
        worktree,
        &["rev-list", "--count", "HEAD", "--not", "--remotes"],
    )?
    .parse()?)
}

/// The branch `origin/HEAD` points to, if the repository has one.
pub fn default_branch(repo: &Path) -> Option<String> {
    let reference = git(
        repo,
        &["symbolic-ref", "--short", "refs/remotes/origin/HEAD"],
    )
    .ok()?;
    reference.strip_prefix("origin/").map(str::to_string)
}

pub fn branch_sha(repo: &Path, branch: &str) -> Result<String> {
    git(repo, &["rev-parse", &format!("refs/heads/{branch}")])
}

/// Removes `worktree` with its uncommitted, untracked and ignored files. A locked
/// worktree still fails.
pub fn remove_worktree(repo: &Path, worktree: &Path) -> Result<()> {
    let worktree = worktree.to_string_lossy();
    git(repo, &["worktree", "remove", "--force", "--", &worktree]).map(drop)
}

pub fn delete_branch(repo: &Path, branch: &str) -> Result<()> {
    git(repo, &["branch", "-D", "--", branch]).map(drop)
}
