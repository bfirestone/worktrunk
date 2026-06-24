use clap::Subcommand;

#[derive(Subcommand)]
pub enum SyncCommand {
    /// Commit and push work to the remote (append-only)
    ///
    /// Stages changes, makes a `wip` commit, and pushes. Never force-pushes.
    Push {
        /// Branch to sync (defaults to current worktree)
        #[arg(add = crate::completion::branch_value_completer(), value_parser = crate::cli::non_empty_branch)]
        branch: Option<String>,

        /// Stage only tracked changes (exclude new files)
        #[arg(long)]
        tracked: bool,

        /// Override the auto-generated wip commit message
        #[arg(short = 'm', long)]
        message: Option<String>,

        /// Output format
        ///
        /// JSON prints structured result to stdout after the push completes.
        #[arg(long, default_value = "text", help_heading = "Automation")]
        format: crate::cli::SwitchFormat,
    },

    /// Fetch and fast-forward the local branch from the remote
    ///
    /// Fast-forward only — never `reset --hard`. Fails if the local branch
    /// has diverged.
    Pull {
        /// Branch to sync (defaults to current worktree)
        #[arg(add = crate::completion::branch_value_completer(), value_parser = crate::cli::non_empty_branch)]
        branch: Option<String>,

        /// Output format
        #[arg(long, default_value = "text", help_heading = "Automation")]
        format: crate::cli::SwitchFormat,
    },
}
