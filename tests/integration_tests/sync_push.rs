//! Integration tests for `wt sync push`.

use crate::common::{TestRepo, repo_with_remote_and_feature};
use rstest::rstest;

/// Run `git log --oneline <branch>` in the bare remote repo and return stdout.
fn remote_log(repo: &TestRepo, branch: &str) -> String {
    let remote = repo
        .remote_path()
        .expect("test fixture has no remote configured");
    let output = repo
        .git_command()
        .current_dir(remote)
        .args(["log", "--oneline", branch])
        .run()
        .unwrap();
    String::from_utf8_lossy(&output.stdout).to_string()
}

/// Push the feature branch from a temporary clone of the remote, creating a
/// commit that is ahead of the local `feature` HEAD.
///
/// Strategy:
/// 1. Clone the remote into a sibling directory.
/// 2. In that clone, create a file and commit it on `feature`.
/// 3. Push back to the shared remote bare repo.
///
/// After this call the remote `feature` has a commit that the local worktree
/// does not — a subsequent local push will be non-fast-forward.
fn advance_remote_feature(repo: &TestRepo) {
    let remote = repo
        .remote_path()
        .expect("test fixture has no remote configured");
    let remote_str = remote.to_str().unwrap();

    // Create a temporary clone next to the existing repos.
    let home = repo.home_path();
    let clone_path = home.join("remote-clone");
    repo.run_git_in(home, &["clone", remote_str, "remote-clone"]);

    // Configure identity in the clone.
    repo.run_git_in(&clone_path, &["config", "user.name", "Test User"]);
    repo.run_git_in(&clone_path, &["config", "user.email", "test@example.com"]);

    // Checkout the feature branch.
    repo.run_git_in(&clone_path, &["checkout", "feature"]);

    // Commit something new.
    std::fs::write(clone_path.join("remote-advance.txt"), "remote advance").unwrap();
    repo.run_git_in(&clone_path, &["add", "remote-advance.txt"]);
    repo.run_git_in(&clone_path, &["commit", "-m", "remote: advance feature"]);

    // Push back to the shared remote.
    repo.run_git_in(&clone_path, &["push", "origin", "feature"]);
}

// ---------------------------------------------------------------------------
// Test 1: happy path — staged change committed and pushed, remote advances
// ---------------------------------------------------------------------------

#[rstest]
fn sync_push_commits_and_pushes(mut repo_with_remote_and_feature: TestRepo) {
    let repo = &mut repo_with_remote_and_feature;
    let wt = repo.worktree_path("feature").to_path_buf();

    // Make a tracked change in the feature worktree.
    std::fs::write(wt.join("file.txt"), "synced edit").unwrap();

    let output = repo
        .wt_command()
        .args(["sync", "push", "--format", "json"])
        .current_dir(&wt)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "wt sync push failed\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    // The remote feature branch now has a wip commit.
    let log = remote_log(repo, "feature");
    assert!(
        log.contains("wip @"),
        "expected wip commit on remote, got: {log}"
    );
}

// ---------------------------------------------------------------------------
// Test 2: --stage tracked with only new (untracked) file → nothing staged
// ---------------------------------------------------------------------------

#[rstest]
fn sync_push_tracked_only_skips_untracked_files(mut repo_with_remote_and_feature: TestRepo) {
    let repo = &mut repo_with_remote_and_feature;
    let wt = repo.worktree_path("feature").to_path_buf();

    // Create an untracked file only — no tracked modifications.
    std::fs::write(wt.join("untracked-new.txt"), "brand new untracked").unwrap();

    // Record the current feature HEAD before the push attempt.
    let pre_sha = repo.head_sha_in(&wt);

    let output = repo
        .wt_command()
        .args(["sync", "push", "--stage", "tracked"])
        .current_dir(&wt)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "wt sync push --stage tracked failed\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    // HEAD must not have moved (no new wip commit was created).
    let post_sha = repo.head_sha_in(&wt);
    assert_eq!(
        pre_sha, post_sha,
        "HEAD should not move when nothing is staged with --stage tracked"
    );
}

// ---------------------------------------------------------------------------
// Test 2b: config-default stage=tracked also skips untracked files
// ---------------------------------------------------------------------------

#[rstest]
fn sync_push_stage_default_from_config(mut repo_with_remote_and_feature: TestRepo) {
    let repo = &mut repo_with_remote_and_feature;
    let wt = repo.worktree_path("feature").to_path_buf();

    // Create an untracked file only — no tracked modifications.
    std::fs::write(
        wt.join("config-default-untracked.txt"),
        "untracked via config",
    )
    .unwrap();

    // Record the current feature HEAD before the push attempt.
    let pre_sha = repo.head_sha_in(&wt);

    // Use --config-set to set sync.stage="tracked" (no --stage flag).
    let output = repo
        .wt_command()
        .args([
            "sync",
            "push",
            "--config-set",
            r#"sync.stage="tracked""#,
            "--format",
            "json",
        ])
        .current_dir(&wt)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "wt sync push with config-set sync.stage=tracked failed\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    // HEAD must not have moved — the config default applied and skipped the untracked file.
    let post_sha = repo.head_sha_in(&wt);
    assert_eq!(
        pre_sha, post_sha,
        "HEAD should not move when sync.stage=tracked is set in config and only untracked files exist"
    );
}

// ---------------------------------------------------------------------------
// Test 3: non-fast-forward push is rejected with a helpful message
// ---------------------------------------------------------------------------

#[rstest]
fn sync_push_rejects_non_fast_forward(mut repo_with_remote_and_feature: TestRepo) {
    let repo = &mut repo_with_remote_and_feature;
    let wt = repo.worktree_path("feature").to_path_buf();

    // First push local feature to remote so an upstream is established.
    repo.run_git_in(&wt, &["push", "-u", "origin", "feature"]);

    // Advance the remote's feature branch ahead of local.
    advance_remote_feature(repo);

    // Make a local commit so local and remote have diverged.
    std::fs::write(wt.join("local.txt"), "local work").unwrap();
    repo.run_git_in(&wt, &["add", "local.txt"]);
    repo.run_git_in(&wt, &["commit", "-m", "local: diverging commit"]);

    // Record the remote tip before the (expected-to-fail) push.
    let remote_tip_before = remote_log(repo, "feature");

    let output = repo
        .wt_command()
        .args(["sync", "push"])
        .current_dir(&wt)
        .output()
        .unwrap();

    assert!(
        !output.status.success(),
        "wt sync push should fail on non-fast-forward, but succeeded"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("wt sync pull"),
        "error message should mention 'wt sync pull', got: {stderr}"
    );
    // Regression guard: the rejection must surface as a clean error, not a
    // panic (a flattened multiline error trips the top-level handler's assert).
    assert!(
        !stderr.contains("panicked"),
        "rejection must not panic; got: {stderr}"
    );

    // The remote tip must be unchanged (nothing was force-pushed).
    let remote_tip_after = remote_log(repo, "feature");
    assert_eq!(
        remote_tip_before, remote_tip_after,
        "remote feature tip must not change after a rejected push"
    );
}
