//! Integration tests for `wt sync pull`.

use crate::common::{TestRepo, repo_with_remote_and_feature};
use rstest::rstest;

/// Push a new commit from a temporary clone of the remote so that the remote
/// `feature` branch is one commit ahead of the local worktree.
///
/// Strategy:
/// 1. Clone the remote into a sibling directory.
/// 2. In that clone, checkout `feature`, create a file, commit, and push back.
///
/// After this call the local `feature` is behind the remote by one commit —
/// a subsequent `wt sync pull` should fast-forward it.
fn advance_remote_feature(repo: &TestRepo) {
    let remote = repo
        .remote_path()
        .expect("test fixture has no remote configured");
    let remote_str = remote.to_str().unwrap();

    let home = repo.home_path();
    let clone_path = home.join("remote-clone");
    repo.run_git_in(home, &["clone", remote_str, "remote-clone"]);

    repo.run_git_in(&clone_path, &["config", "user.name", "Test User"]);
    repo.run_git_in(&clone_path, &["config", "user.email", "test@example.com"]);

    repo.run_git_in(&clone_path, &["checkout", "feature"]);

    std::fs::write(clone_path.join("remote-advance.txt"), "remote advance").unwrap();
    repo.run_git_in(&clone_path, &["add", "remote-advance.txt"]);
    repo.run_git_in(&clone_path, &["commit", "-m", "remote: advance feature"]);

    repo.run_git_in(&clone_path, &["push", "origin", "feature"]);
}

// ---------------------------------------------------------------------------
// Test 1: happy path — remote is ahead, local fast-forwards
// ---------------------------------------------------------------------------

#[rstest]
fn sync_pull_fast_forwards(mut repo_with_remote_and_feature: TestRepo) {
    let repo = &mut repo_with_remote_and_feature;
    let wt = repo.worktree_path("feature").to_path_buf();

    // Push local feature to the remote so an upstream is established, then
    // advance the remote by one commit from a separate clone.
    repo.run_git_in(&wt, &["push", "-u", "origin", "feature"]);
    advance_remote_feature(repo);

    // Record local HEAD before the pull.
    let pre_sha = repo.head_sha_in(&wt);

    let output = repo
        .wt_command()
        .args(["sync", "pull", "--format", "json"])
        .current_dir(&wt)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "wt sync pull failed\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("fast_forwarded"),
        "expected 'fast_forwarded' in JSON output, got: {stdout}"
    );

    // Local HEAD must have advanced.
    let post_sha = repo.head_sha_in(&wt);
    assert_ne!(
        pre_sha, post_sha,
        "HEAD should advance after a fast-forward pull"
    );
}

// ---------------------------------------------------------------------------
// Test 2: up-to-date — remote has nothing new, command succeeds and no-ops
// ---------------------------------------------------------------------------

#[rstest]
fn sync_pull_up_to_date(mut repo_with_remote_and_feature: TestRepo) {
    let repo = &mut repo_with_remote_and_feature;
    let wt = repo.worktree_path("feature").to_path_buf();

    // Push local feature to the remote — local and remote are now in sync.
    repo.run_git_in(&wt, &["push", "-u", "origin", "feature"]);

    let pre_sha = repo.head_sha_in(&wt);

    let output = repo
        .wt_command()
        .args(["sync", "pull", "--format", "json"])
        .current_dir(&wt)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "wt sync pull (up-to-date) failed\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("up_to_date"),
        "expected 'up_to_date' in JSON output, got: {stdout}"
    );

    // Local HEAD must be unchanged.
    let post_sha = repo.head_sha_in(&wt);
    assert_eq!(
        pre_sha, post_sha,
        "HEAD should not move when already up to date"
    );
}

// ---------------------------------------------------------------------------
// Test 3: diverged — local has an unpushed commit AND remote is ahead.
//          wt sync pull must fail and leave local HEAD unchanged.
// ---------------------------------------------------------------------------

#[rstest]
fn sync_pull_diverged_fails_safely(mut repo_with_remote_and_feature: TestRepo) {
    let repo = &mut repo_with_remote_and_feature;
    let wt = repo.worktree_path("feature").to_path_buf();

    // Establish upstream.
    repo.run_git_in(&wt, &["push", "-u", "origin", "feature"]);

    // Advance the remote by one commit (from a separate clone).
    advance_remote_feature(repo);

    // Make a local commit so local and remote have diverged.
    std::fs::write(wt.join("local-only.txt"), "local work").unwrap();
    repo.run_git_in(&wt, &["add", "local-only.txt"]);
    repo.run_git_in(&wt, &["commit", "-m", "local: diverging commit"]);

    let pre_sha = repo.head_sha_in(&wt);

    let output = repo
        .wt_command()
        .args(["sync", "pull"])
        .current_dir(&wt)
        .output()
        .unwrap();

    assert!(
        !output.status.success(),
        "wt sync pull should fail on diverged branches, but succeeded"
    );
    // Regression guard: divergence must surface as a clean error, not a panic
    // (a flattened multiline error trips the top-level handler's assert).
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("panicked"),
        "diverged pull must not panic; got: {stderr}"
    );

    // Local HEAD must be unchanged — no data loss.
    let post_sha = repo.head_sha_in(&wt);
    assert_eq!(
        pre_sha, post_sha,
        "HEAD must not change after a failed (diverged) sync pull"
    );
}
