//! Data-safety invariant: the `wt sync` code path must never force-push or
//! hard-reset. Enforced by scanning the module source.

use std::fs;
use std::path::Path;

#[test]
fn sync_module_has_no_destructive_git_commands() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/commands/sync");
    let mut sources = String::new();
    for entry in fs::read_dir(&dir).expect("sync module dir exists") {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|e| e == "rs") {
            sources.push_str(&fs::read_to_string(&path).unwrap());
        }
    }
    // Append-only model: no force-push, no hard reset anywhere in sync.
    assert!(
        !sources.contains("--force"),
        "wt sync must never force-push (append-only invariant)"
    );
    assert!(
        !sources.contains("reset"),
        "wt sync must never use git reset (use merge --ff-only instead)"
    );
}
