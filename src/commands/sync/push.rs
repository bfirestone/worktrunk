use crate::commands::sync::SyncPushResult;

pub fn sync_push(
    _branch: Option<String>,
    _tracked: bool,
    _message: Option<String>,
) -> anyhow::Result<SyncPushResult> {
    anyhow::bail!("wt sync push: not yet implemented")
}
