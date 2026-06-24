+++
title = "wt sync"
description = "Sync work across machines (append-only). Push commits + pushes to the remote; pull fast-forwards from it. No force-push, no reset --hard. Squash the wip stack later with wt merge."
weight = 18

[extra]
group = "Commands"
+++

<!-- ⚠️ AUTO-GENERATED from `wt sync --help-page` — edit src/cli/mod.rs to update -->

Sync work across machines (append-only). Push commits + pushes to the remote; pull fast-forwards from it. No force-push, no reset --hard. Squash the wip stack later with wt merge.

`wt sync` moves in-progress work between machines through the git remote, append-only — no history rewriting. <span class="badge-experimental"></span>

## Examples

```bash
wt sync push
wt sync pull
```

On machine A, `wt sync push` stages changes (default: `git add -A`; use `--stage tracked` for tracked-only), makes a `wip @ <hostname> — <timestamp>` commit, and pushes (setting upstream on the first push). On machine B, `wt sync pull` fetches and fast-forwards. Divergence or conflicting local changes fail safely rather than clobbering work.

## Command reference

{% terminal() %}
wt sync - Sync work across machines (append-only)

Push commits + pushes to the remote; pull fast-forwards from it. No force-push, no <b>reset --hard</b>.
Squash the wip stack later with <b>wt merge</b>.

Usage: <b><span class=c>wt sync</span></b> <span class=c>[OPTIONS]</span> <span class=c>&lt;COMMAND&gt;</span>

<b><span class=g>Commands:</span></b>
  <b><span class=c>push</span></b>  Commit and push work to the remote (append-only)
  <b><span class=c>pull</span></b>  Fetch and fast-forward the local branch from the remote

<b><span class=g>Options:</span></b>
  <b><span class=c>-h</span></b>, <b><span class=c>--help</span></b>
          Print help (see a summary with &#39;-h&#39;)

<b><span class=g>Global Options:</span></b>
  <b><span class=c>-C</span></b><span class=c> &lt;path&gt;</span>
          Working directory for this command

      <b><span class=c>--config</span></b><span class=c> &lt;path&gt;</span>
          User config file path

      <b><span class=c>--config-set</span></b><span class=c> &lt;toml&gt;</span>
          Override config with inline TOML, e.g. --config-set list.full=true (repeatable)

  <b><span class=c>-v</span></b>, <b><span class=c>--verbose</span></b><span class=c>...</span>
          Verbose output (-v: info logs + hook/alias template variables on stderr; -vv: also debug
          logs and raw subprocess output written to .git/wt/logs/). Set WORKTRUNK_VERBOSE=0|1|2 to
          apply the same level everywhere — including shell completion, which no flag can reach

  <b><span class=c>-y</span></b>, <b><span class=c>--yes</span></b>
          Skip approval prompts
{% end %}

<!-- END AUTO-GENERATED -->
