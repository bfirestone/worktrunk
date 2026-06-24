use color_print::cformat;
use worktrunk::git::{ErrorExt, Repository};
use worktrunk::styling::{eprintln, info_message, progress_message, success_message};

use crate::commands::sync::{SyncPullOutcome, SyncPullResult, resolve_sync_remote};

pub fn sync_pull(branch: Option<String>) -> anyhow::Result<SyncPullResult> {
    let repo = Repository::current()?;
    let branch = match branch {
        Some(b) => b,
        None => repo.require_current_branch("sync pull")?,
    };
    let remote = resolve_sync_remote(&repo, &branch)?;

    // 1. Fetch the branch from the remote.
    eprintln!(
        "{}",
        progress_message(cformat!(
            "Fetching <bold>{branch}</> from <bold>{remote}</>..."
        ))
    );
    repo.run_command(&["fetch", &remote, &branch])
        .map_err(|e| {
            anyhow::anyhow!(
                "Failed to fetch {branch} from {remote}: {}",
                e.display_message()
            )
        })?;

    let remote_ref = format!("{remote}/{branch}");

    // 2. How many commits is the remote ahead? 0 → already up to date.
    let commits_pulled = repo
        .run_command(&["rev-list", "--count", &format!("HEAD..{remote_ref}")])
        .ok()
        .and_then(|s| s.trim().parse::<usize>().ok())
        .unwrap_or(0);

    if commits_pulled == 0 {
        eprintln!(
            "{}",
            info_message(cformat!("Already up to date with <bold>{remote_ref}</>"))
        );
        return Ok(SyncPullResult {
            branch,
            remote,
            outcome: SyncPullOutcome::UpToDate,
            commits_pulled: 0,
        });
    }

    // 3. Advance with the safe primitive: merge with ff-only. It refuses (and
    //    changes nothing) when the local branch has diverged or when uncommitted
    //    changes would be overwritten — surface git's message rather than
    //    clobbering anything.
    repo.run_command(&["merge", "--ff-only", &remote_ref])
        .map_err(|e| {
            anyhow::anyhow!(
                "Cannot fast-forward {branch} to {remote_ref} — the local branch \
                 has diverged (you may have unpushed commits) or has conflicting \
                 uncommitted changes. Run `wt sync push` or reconcile manually.\n{}",
                e.display_message()
            )
        })?;

    eprintln!(
        "{}",
        success_message(cformat!(
            "Fast-forwarded <bold>{branch}</> to <bold>{remote_ref}</> ({commits_pulled} commits)"
        ))
    );

    Ok(SyncPullResult {
        branch,
        remote,
        outcome: SyncPullOutcome::FastForwarded,
        commits_pulled,
    })
}
