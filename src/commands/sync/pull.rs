use crate::commands::sync::SyncPullResult;

pub fn sync_pull(_branch: Option<String>) -> anyhow::Result<SyncPullResult> {
    anyhow::bail!("wt sync pull: not yet implemented")
}
