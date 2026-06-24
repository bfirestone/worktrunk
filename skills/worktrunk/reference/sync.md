# wt sync

Sync work across machines (append-only). Push commits + pushes to the remote; pull fast-forwards from it. No force-push, no reset --hard. Squash the wip stack later with wt merge.

`wt sync` moves in-progress work between machines through the git remote, append-only — no history rewriting. [experimental]

## Examples

```bash
wt sync push
wt sync pull
```

On machine A, `wt sync push` stages changes (default: `git add -A`; use `--stage tracked` for tracked-only), makes a `wip @ <hostname> — <timestamp>` commit, and pushes (setting upstream on the first push). On machine B, `wt sync pull` fetches and fast-forwards. Divergence or conflicting local changes fail safely rather than clobbering work.

## Command reference

```
wt sync - Sync work across machines (append-only)

Push commits + pushes to the remote; pull fast-forwards from it. No force-push, no reset --hard.
Squash the wip stack later with wt merge.

Usage: wt sync [OPTIONS] <COMMAND>

Commands:
  push  Commit and push work to the remote (append-only)
  pull  Fetch and fast-forward the local branch from the remote

Options:
  -h, --help
          Print help (see a summary with '-h')

Global Options:
  -C <path>
          Working directory for this command

      --config <path>
          User config file path

      --config-set <toml>
          Override config with inline TOML, e.g. --config-set list.full=true (repeatable)

  -v, --verbose...
          Verbose output (-v: info logs + hook/alias template variables on stderr; -vv: also debug
          logs and raw subprocess output written to .git/wt/logs/). Set WORKTRUNK_VERBOSE=0|1|2 to
          apply the same level everywhere — including shell completion, which no flag can reach

  -y, --yes
          Skip approval prompts
```
