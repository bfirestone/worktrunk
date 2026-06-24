//! `wt sync` — cross-machine append-only sync.
//!
//! Append-only model: `push` commits + pushes (never force), `pull`
//! fast-forwards only (never rewinds the local branch destructively). The wip
//! stack is squashed later by `wt merge`. See
//! docs/plans/2026-06-24-wt-sync-cross-machine.md.
//!
//! The data-safety guard (`tests/integration_tests/sync_guard.rs`) scans this
//! module's source for the destructive-git substrings it forbids, so prose
//! here must describe the invariant without naming those commands verbatim.

use worktrunk::git::Repository;

mod pull;
mod push;

pub use pull::sync_pull;
pub use push::sync_push;

/// Result of `wt sync push`, returned for JSON output.
pub struct SyncPushResult {
    pub branch: String,
    pub remote: String,
    /// `Some(sha)` if a wip commit was created this run; `None` if nothing was staged.
    pub committed: Option<String>,
    pub outcome: SyncPushOutcome,
    /// Commits pushed to the remote (0 when already up to date).
    pub commits_pushed: usize,
}

pub enum SyncPushOutcome {
    Pushed,
    UpToDate,
}

/// Result of `wt sync pull`, returned for JSON output.
pub struct SyncPullResult {
    pub branch: String,
    pub remote: String,
    pub outcome: SyncPullOutcome,
    /// Commits fast-forwarded into the local branch (0 when already up to date).
    pub commits_pulled: usize,
}

pub enum SyncPullOutcome {
    FastForwarded,
    UpToDate,
}

/// Build the auto-generated wip commit message:
/// `wip @ <hostname> — <RFC3339 timestamp>`.
///
/// Diagnostic only (squashed at `wt merge`); the hostname tag identifies which
/// machine made each checkpoint.
pub fn wip_message() -> String {
    let host = gethostname::gethostname().to_string_lossy().into_owned();
    let ts = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    format!("wip @ {host} — {ts}")
}

/// Resolve the remote to sync `branch` against. Reuses the branch's configured
/// push remote (`branch.<name>.pushRemote` → `remote.pushDefault` →
/// `branch.<name>.remote`), falling back to `origin`.
pub fn resolve_sync_remote(repo: &Repository, branch: &str) -> anyhow::Result<String> {
    Ok(repo
        .branch(branch)
        .push_remote()
        .unwrap_or_else(|| "origin".to_string()))
}
