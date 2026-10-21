//! End-to-end tests: a temporary T3 home with a minimal state database and real
//! git repositories with worktrees.

use rusqlite::{Connection, params};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

const OLD: &str = "2026-01-01T00:00:00.000Z";

fn git(dir: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

struct Fixture {
    dir: TempDir,
    db: Connection,
}

impl Fixture {
    /// A project `demo` cloned from a local origin, and an empty T3 database.
    fn new() -> Fixture {
        let dir = TempDir::new().unwrap();
        let origin = dir.path().join("origin.git");
        git(
            dir.path(),
            &[
                "init",
                "--quiet",
                "--bare",
                "-b",
                "main",
                origin.to_str().unwrap(),
            ],
        );
        git(
            dir.path(),
            &["clone", "--quiet", origin.to_str().unwrap(), "demo"],
        );
        let root = dir.path().join("demo");
        git(&root, &["config", "user.email", "test@example.com"]);
        git(&root, &["config", "user.name", "Test"]);
        git(
            &root,
            &["commit", "--quiet", "--allow-empty", "-m", "initial"],
        );
        git(&root, &["push", "--quiet", "origin", "main"]);
        git(&root, &["remote", "set-head", "origin", "main"]);

        std::fs::create_dir_all(dir.path().join("t3/userdata")).unwrap();
        let db = Connection::open(dir.path().join("t3/userdata/statev2.sqlite")).unwrap();
        db.execute_batch(
            "CREATE TABLE projection_projects (project_id TEXT, title TEXT, workspace_root TEXT);
             CREATE TABLE orchestration_v2_projection_threads (
               thread_id TEXT, project_id TEXT, title TEXT, created_at TEXT, payload_json TEXT);
             CREATE TABLE orchestration_v2_projection_runs (
               thread_id TEXT, requested_at TEXT, completed_at TEXT, payload_json TEXT);
             CREATE TABLE orchestration_v2_projection_messages (
               thread_id TEXT, role TEXT, updated_at TEXT);",
        )
        .unwrap();
        db.execute(
            "INSERT INTO projection_projects VALUES ('p1', 'demo', ?1)",
            params![root.to_str().unwrap()],
        )
        .unwrap();
        Fixture { dir, db }
    }

    fn root(&self) -> PathBuf {
        self.dir.path().join("demo")
    }

    /// Adds a worktree on a new branch and pushes it, so it has no unpushed commits.
    fn worktree(&self, name: &str) -> PathBuf {
        let path = self.dir.path().join("t3/worktrees/demo").join(name);
        git(
            &self.root(),
            &[
                "worktree",
                "add",
                "--quiet",
                "-b",
                name,
                path.to_str().unwrap(),
            ],
        );
        git(&path, &["push", "--quiet", "origin", name]);
        path
    }

    fn thread(&self, id: &str, worktree: &Path, created_at: &str) {
        let payload = serde_json::json!({ "worktreePath": worktree, "settledAt": created_at });
        self.db
            .execute(
                "INSERT INTO orchestration_v2_projection_threads VALUES (?1, 'p1', ?1, ?2, ?3)",
                params![id, created_at, payload.to_string()],
            )
            .unwrap();
    }

    fn run(&self, args: &[&str]) -> (i32, Value) {
        let output = Command::new(env!("CARGO_BIN_EXE_t3-worktree-cleanup"))
            .args(args)
            .arg("--base-dir")
            .arg(self.dir.path().join("t3"))
            .output()
            .unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        let json = serde_json::from_str(&stdout).unwrap_or(Value::Null);
        (output.status.code().unwrap(), json)
    }

    fn plan(&self) -> Value {
        let (code, plan) = self.run(&["plan"]);
        assert_eq!(code, 0);
        plan
    }
}

fn now() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

fn statuses(plan: &Value) -> Vec<(String, String)> {
    plan["worktrees"]
        .as_array()
        .unwrap()
        .iter()
        .map(|w| {
            let path = w["path"].as_str().unwrap();
            let name = path.rsplit('/').next().unwrap().to_string();
            (name, w["status"].as_str().unwrap().to_string())
        })
        .collect()
}

#[test]
fn plan_lists_inactive_worktrees_and_counts_recent_ones() {
    let fixture = Fixture::new();
    let old = fixture.worktree("old");
    let fresh = fixture.worktree("fresh");
    fixture.thread("t1", &old, OLD);
    fixture.thread("t2", &fresh, &now());
    fixture.thread(
        "t3",
        &fixture.dir.path().join("t3/worktrees/demo/gone"),
        OLD,
    );

    let plan = fixture.plan();

    assert_eq!(statuses(&plan), [("old".into(), "candidate".into())]);
    assert_eq!(plan["candidates"], 1);
    assert_eq!(plan["recent"], 1);
    let worktree = &plan["worktrees"][0];
    assert_eq!(worktree["project"], "demo");
    assert_eq!(worktree["branch"], "old");
    assert_eq!(worktree["deleteBranch"], true);
    assert_eq!(worktree["threads"][0]["id"], "t1");
}

#[test]
fn shared_worktree_uses_the_latest_activity_of_all_threads() {
    let fixture = Fixture::new();
    let shared = fixture.worktree("shared");
    fixture.thread("root", &shared, OLD);
    fixture.thread("fork", &shared, OLD);
    assert_eq!(
        statuses(&fixture.plan()),
        [("shared".into(), "candidate".into())]
    );

    fixture.thread("second-root", &shared, &now());
    let plan = fixture.plan();
    assert_eq!(statuses(&plan), []);
    assert_eq!(plan["recent"], 1);
}

#[test]
fn runs_and_user_messages_count_as_activity() {
    let fixture = Fixture::new();
    let by_run = fixture.worktree("by-run");
    let by_message = fixture.worktree("by-message");
    fixture.thread("run", &by_run, OLD);
    fixture.thread("message", &by_message, OLD);
    fixture
        .db
        .execute(
            "INSERT INTO orchestration_v2_projection_runs VALUES ('run', ?1, NULL, ?2)",
            params![OLD, serde_json::json!({ "startedAt": now() }).to_string()],
        )
        .unwrap();
    fixture
        .db
        .execute(
            "INSERT INTO orchestration_v2_projection_messages VALUES ('message', 'user', ?1)",
            params![now()],
        )
        .unwrap();

    assert_eq!(fixture.plan()["recent"], 2);
}

#[test]
fn plan_skips_worktrees_with_local_work() {
    let fixture = Fixture::new();
    let dirty = fixture.worktree("dirty");
    let unpushed = fixture.worktree("unpushed");
    std::fs::write(dirty.join("notes.txt"), "draft").unwrap();
    git(
        &unpushed,
        &["commit", "--quiet", "--allow-empty", "-m", "local"],
    );
    fixture.thread("t1", &dirty, OLD);
    fixture.thread("t2", &unpushed, OLD);

    let plan = fixture.plan();

    assert_eq!(plan["skipped"], 2);
    let reasons: Vec<&Value> = plan["worktrees"]
        .as_array()
        .unwrap()
        .iter()
        .map(|w| &w["reasons"])
        .collect();
    assert!(reasons.contains(&&serde_json::json!(["uncommitted or untracked changes"])));
    assert!(reasons.contains(&&serde_json::json!(["1 commits not on any remote"])));
}

#[test]
fn project_filter_matches_title_or_root() {
    let fixture = Fixture::new();
    let old = fixture.worktree("old");
    fixture.thread("t1", &old, OLD);

    for project in ["demo", "p1", fixture.root().to_str().unwrap()] {
        let (_, plan) = fixture.run(&["plan", "--project", project]);
        assert_eq!(plan["candidates"], 1, "--project {project}");
    }
    let (_, plan) = fixture.run(&["plan", "--project", "other"]);
    assert_eq!(plan["candidates"], 0);
}

#[test]
fn older_than_sets_the_cutoff() {
    let fixture = Fixture::new();
    let old = fixture.worktree("old");
    let yesterday = (chrono::Utc::now() - chrono::TimeDelta::hours(30))
        .to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    fixture.thread("t1", &old, &yesterday);

    assert_eq!(fixture.run(&["plan"]).1["candidates"], 0);
    assert_eq!(
        fixture.run(&["plan", "--older-than", "1d"]).1["candidates"],
        1
    );
    assert_eq!(
        fixture.run(&["plan", "--older-than", "36h"]).1["candidates"],
        0
    );
}

#[test]
fn apply_deletes_the_worktree_and_its_branch() {
    let fixture = Fixture::new();
    let old = fixture.worktree("old");
    fixture.thread("t1", &old, OLD);
    let sha = git(&fixture.root(), &["rev-parse", "old"]);

    let (code, outcomes) = fixture.run(&["apply", old.to_str().unwrap()]);

    assert_eq!(code, 0);
    assert_eq!(outcomes[0]["status"], "deleted");
    assert_eq!(outcomes[0]["branch"]["name"], "old");
    assert_eq!(outcomes[0]["branch"]["sha"], sha.as_str());
    assert_eq!(outcomes[0]["branch"]["deleted"], true);
    assert!(!old.exists());
    assert_eq!(git(&fixture.root(), &["branch", "--list", "old"]), "");
}

#[test]
fn apply_keeps_the_default_branch() {
    let fixture = Fixture::new();
    let path = fixture.dir.path().join("t3/worktrees/demo/default");
    git(&fixture.root(), &["checkout", "--quiet", "--detach"]);
    git(
        &fixture.root(),
        &["worktree", "add", "--quiet", path.to_str().unwrap(), "main"],
    );
    fixture.thread("t1", &path, OLD);

    assert_eq!(fixture.plan()["worktrees"][0]["deleteBranch"], false);
    let (code, outcomes) = fixture.run(&["apply", path.to_str().unwrap()]);

    assert_eq!(code, 0);
    assert_eq!(outcomes[0]["status"], "deleted");
    assert_eq!(outcomes[0]["branch"]["deleted"], false);
    assert_eq!(outcomes[0]["branch"]["note"], "default branch is kept");
    assert_eq!(git(&fixture.root(), &["branch", "--list", "main"]), "main");
}

#[test]
fn apply_skips_local_work_unless_forced() {
    let fixture = Fixture::new();
    let dirty = fixture.worktree("dirty");
    std::fs::write(dirty.join("notes.txt"), "draft").unwrap();
    fixture.thread("t1", &dirty, OLD);

    let (code, outcomes) = fixture.run(&["apply", dirty.to_str().unwrap()]);
    assert_eq!(code, 0);
    assert_eq!(outcomes[0]["status"], "skipped");
    assert!(dirty.exists());

    let (code, outcomes) = fixture.run(&["apply", "--force", dirty.to_str().unwrap()]);
    assert_eq!(code, 0);
    assert_eq!(outcomes[0]["status"], "deleted");
    assert!(!dirty.exists());
}

#[test]
fn apply_rechecks_activity_and_rejects_unknown_paths() {
    let fixture = Fixture::new();
    let fresh = fixture.worktree("fresh");
    fixture.thread("t1", &fresh, &now());

    let (code, outcomes) = fixture.run(&["apply", fresh.to_str().unwrap(), "/nowhere"]);

    assert_eq!(code, 1);
    assert_eq!(outcomes[0]["status"], "skipped");
    assert_eq!(
        outcomes[0]["reasons"][0],
        "active within the selected period"
    );
    assert_eq!(outcomes[1]["status"], "failed");
    assert!(fresh.exists());
}

#[test]
fn apply_reports_git_failures_and_continues() {
    let fixture = Fixture::new();
    let locked = fixture.worktree("locked");
    let old = fixture.worktree("old");
    git(
        &fixture.root(),
        &["worktree", "lock", locked.to_str().unwrap()],
    );
    fixture.thread("t1", &locked, OLD);
    fixture.thread("t2", &old, OLD);

    let (code, outcomes) = fixture.run(&["apply", locked.to_str().unwrap(), old.to_str().unwrap()]);

    assert_eq!(code, 1);
    assert_eq!(outcomes[0]["status"], "failed");
    assert_eq!(outcomes[1]["status"], "deleted");
}

#[test]
fn missing_database_is_an_error() {
    let dir = TempDir::new().unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_t3-worktree-cleanup"))
        .args(["plan", "--base-dir"])
        .arg(dir.path())
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("cannot open T3 state database"));
}

#[test]
fn plan_reports_the_size_on_disk() {
    let fixture = Fixture::new();
    let old = fixture.worktree("old");
    std::fs::create_dir(old.join("data")).unwrap();
    std::fs::write(old.join("data/blob"), vec![0u8; 5000]).unwrap();
    git(&old, &["add", "data"]);
    git(&old, &["commit", "--quiet", "-m", "blob"]);
    git(&old, &["push", "--quiet", "origin", "old"]);
    fixture.thread("t1", &old, OLD);
    let git_file = std::fs::metadata(old.join(".git")).unwrap().len();

    let plan = fixture.plan();

    assert_eq!(plan["worktrees"][0]["sizeBytes"], 5000 + git_file);
    assert_eq!(plan["candidateBytes"], 5000 + git_file);
}

#[test]
fn plan_keeps_the_branch_origin_head_points_to() {
    let fixture = Fixture::new();
    let trunk = fixture.worktree("trunk");
    git(&fixture.root(), &["remote", "set-head", "origin", "trunk"]);
    fixture.thread("t1", &trunk, OLD);

    assert_eq!(fixture.plan()["worktrees"][0]["deleteBranch"], false);
}

#[test]
fn base_dir_defaults_to_t3code_home_then_home() {
    let fixture = Fixture::new();
    let old = fixture.worktree("old");
    fixture.thread("t1", &old, OLD);
    let t3_home = fixture.dir.path().join("t3");
    let home = TempDir::new().unwrap();
    std::os::unix::fs::symlink(&t3_home, home.path().join(".t3")).unwrap();
    let plan = |command: &mut Command| -> Value {
        let output = command.arg("plan").output().unwrap();
        serde_json::from_slice(&output.stdout).unwrap()
    };
    let binary = env!("CARGO_BIN_EXE_t3-worktree-cleanup");

    let from_t3_home = plan(Command::new(binary).env("T3CODE_HOME", &t3_home));
    let from_home = plan(
        Command::new(binary)
            .env_remove("T3CODE_HOME")
            .env("HOME", home.path()),
    );

    assert_eq!(from_t3_home["candidates"], 1);
    assert_eq!(from_home["candidates"], 1);
}

#[test]
fn force_does_not_override_failed_checks() {
    let fixture = Fixture::new();
    let broken = fixture.dir.path().join("t3/worktrees/demo/broken");
    std::fs::create_dir_all(&broken).unwrap();
    std::fs::write(broken.join(".git"), "gitdir: /nowhere").unwrap();
    fixture.thread("t1", &broken, OLD);

    let (code, outcomes) = fixture.run(&["apply", "--force", broken.to_str().unwrap()]);

    assert_eq!(code, 0);
    assert_eq!(outcomes[0]["status"], "skipped");
    assert!(
        outcomes[0]["reasons"][0]
            .as_str()
            .unwrap()
            .contains("git status")
    );
    assert!(broken.exists());
}

#[test]
fn project_filter_keeps_activity_from_other_projects() {
    let fixture = Fixture::new();
    let shared = fixture.worktree("shared");
    fixture.thread("t1", &shared, OLD);
    fixture
        .db
        .execute(
            "INSERT INTO projection_projects VALUES ('p2', 'other', '/elsewhere')",
            [],
        )
        .unwrap();
    let payload = serde_json::json!({ "worktreePath": shared });
    fixture
        .db
        .execute(
            "INSERT INTO orchestration_v2_projection_threads VALUES ('t2', 'p2', 't2', ?1, ?2)",
            params![now(), payload.to_string()],
        )
        .unwrap();

    let (_, plan) = fixture.run(&["plan", "--project", "demo"]);

    assert_eq!(plan["candidates"], 0);
    assert_eq!(plan["recent"], 1);
}
