//! Reads threads and their worktree paths from T3's state database.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::{Connection, OpenFlags};
use std::path::{Path, PathBuf};

/// A T3 thread that references a worktree.
#[derive(Debug, Clone)]
pub struct ThreadRow {
    pub thread_id: String,
    pub title: String,
    pub project_id: String,
    pub project_title: String,
    pub project_root: PathBuf,
    pub worktree_path: PathBuf,
    pub settled_at: Option<String>,
    pub activity_at: DateTime<Utc>,
}

/// Activity follows T3's own cleanup clock: thread creation, the latest user
/// message and run request, start and completion times. Renames, settling and
/// PR refreshes do not count. Timestamps are ISO-8601 UTC strings, so string
/// `max` orders them correctly.
const THREADS_QUERY: &str = "
WITH runs AS (
  SELECT thread_id,
         max(max(requested_at,
                 coalesce(json_extract(payload_json, '$.startedAt'), ''),
                 coalesce(completed_at, ''))) AS run_at
  FROM orchestration_v2_projection_runs
  GROUP BY thread_id
),
user_messages AS (
  SELECT thread_id, max(updated_at) AS message_at
  FROM orchestration_v2_projection_messages
  WHERE role = 'user'
  GROUP BY thread_id
)
SELECT t.thread_id,
       t.title,
       t.project_id,
       p.title,
       p.workspace_root,
       json_extract(t.payload_json, '$.worktreePath'),
       json_extract(t.payload_json, '$.settledAt'),
       max(t.created_at, coalesce(m.message_at, ''), coalesce(r.run_at, ''))
FROM orchestration_v2_projection_threads t
JOIN projection_projects p ON p.project_id = t.project_id
LEFT JOIN runs r ON r.thread_id = t.thread_id
LEFT JOIN user_messages m ON m.thread_id = t.thread_id
WHERE json_extract(t.payload_json, '$.worktreePath') IS NOT NULL
";

/// Returns every thread with a worktree path, including deleted and archived ones.
pub fn read_threads(base_dir: &Path) -> Result<Vec<ThreadRow>> {
    let db = base_dir.join("userdata").join("statev2.sqlite");
    let connection = Connection::open_with_flags(&db, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .with_context(|| format!("cannot open T3 state database {}", db.display()))?;
    let mut statement = connection
        .prepare(THREADS_QUERY)
        .context("T3 state database has an unexpected schema")?;
    let rows = statement.query_map([], |row| {
        Ok((
            ThreadRow {
                thread_id: row.get(0)?,
                title: row.get(1)?,
                project_id: row.get(2)?,
                project_title: row.get(3)?,
                project_root: PathBuf::from(row.get::<_, String>(4)?),
                worktree_path: PathBuf::from(row.get::<_, String>(5)?),
                settled_at: row.get(6)?,
                activity_at: DateTime::<Utc>::MIN_UTC,
            },
            row.get::<_, String>(7)?,
        ))
    })?;
    rows.map(|row| {
        let (mut thread, activity_at) = row?;
        thread.activity_at = DateTime::parse_from_rfc3339(&activity_at)
            .with_context(|| {
                format!(
                    "thread {} has an invalid timestamp {activity_at:?}",
                    thread.thread_id
                )
            })?
            .with_timezone(&Utc);
        Ok(thread)
    })
    .collect()
}
