use anyhow::Context;
use color_print::cformat;
use worktrunk::git::{ErrorExt, Repository};
use worktrunk::styling::{eprintln, info_message, progress_message, success_message};

use crate::commands::sync::{SyncPushOutcome, SyncPushResult, resolve_sync_remote, wip_message};

pub fn sync_push(
    branch: Option<String>,
    tracked: bool,
    message: Option<String>,
) -> anyhow::Result<SyncPushResult> {
    let repo = Repository::current()?;
    let branch = match branch {
        Some(b) => b,
        None => repo.require_current_branch("sync push")?,
    };
    let remote = resolve_sync_remote(&repo, &branch)?;

    // 1. Stage changes. -A stages all (incl. new files); -u only tracked files.
    let add_flag = if tracked { "-u" } else { "-A" };
    repo.run_command(&["add", add_flag])
        .context("Failed to stage changes")?;

    // 2. Commit only when something is actually staged.
    //    `diff --cached --quiet` exits 0 when the index is clean,
    //    non-zero when there are staged changes — so run_command_check returns
    //    false (i.e., "there ARE staged changes") when the index is dirty.
    let has_staged = !repo.run_command_check(&["diff", "--cached", "--quiet"])?;
    let committed = if has_staged {
        let msg = message.unwrap_or_else(wip_message);
        eprintln!("{}", progress_message("Committing wip checkpoint..."));
        repo.run_command(&["commit", "-m", &msg])
            .context("Failed to create wip commit")?;
        let sha = repo.run_command(&["rev-parse", "HEAD"])?.trim().to_string();
        Some(sha)
    } else {
        None
    };

    // 3. Count commits ahead of upstream for reporting.
    //    If there is no upstream yet, count all commits on the branch so that
    //    the first push always reports > 0 commits pushed.
    let has_upstream = repo.branch(&branch).upstream()?.is_some();
    let commits_pushed = if has_upstream {
        repo.run_command(&["rev-list", "--count", "@{upstream}..HEAD"])
            .ok()
            .and_then(|s| s.trim().parse::<usize>().ok())
            .unwrap_or(0)
    } else {
        repo.run_command(&["rev-list", "--count", "HEAD"])
            .ok()
            .and_then(|s| s.trim().parse::<usize>().ok())
            .unwrap_or(1)
    };

    // 4. Push (append-only). Set upstream on the first push with -u.
    eprintln!(
        "{}",
        progress_message(cformat!(
            "Pushing <bold>{branch}</> to <bold>{remote}</>..."
        ))
    );
    let push_args: Vec<&str> = if has_upstream {
        vec!["push", &remote, &branch]
    } else {
        vec!["push", "-u", &remote, &branch]
    };
    repo.run_command(&push_args).map_err(|e| {
        // A non-fast-forward rejection means the remote has commits we don't have.
        anyhow::anyhow!(
            "Push to {remote}/{branch} was rejected — the remote has commits you \
             don't have. Run `wt sync pull` first.\n{}",
            e.display_message()
        )
    })?;

    let outcome = if commits_pushed > 0 {
        eprintln!(
            "{}",
            success_message(cformat!("Pushed <bold>{branch}</> to <bold>{remote}</>"))
        );
        SyncPushOutcome::Pushed
    } else {
        eprintln!(
            "{}",
            info_message(cformat!(
                "Already up to date with <bold>{remote}/{branch}</>"
            ))
        );
        SyncPushOutcome::UpToDate
    };

    Ok(SyncPushResult {
        branch,
        remote,
        committed,
        outcome,
        commits_pushed,
    })
}
