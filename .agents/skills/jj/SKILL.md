---
name: jj
description: Use for any version-control task in repositories managed by Jujutsu (`jj`) or colocated `.jj` + `.git` repos. Trigger when the user asks for status, history rewriting, commit description updates, bookmark/branch management, push/fetch workflows, conflict handling, undo/recovery, or Git-to-Jujutsu migration. Prefer `jj` commands for repository mutations.
---

# Jujutsu (`jj`) Workflow Skill

Execute version-control tasks with `jj` first. Keep commands non-interactive unless the user explicitly asks for an interactive workflow.

## Detect Repository Mode

1. Run `jj root` to confirm Jujutsu context.
2. Treat the repository as Jujutsu-managed when `.jj/` exists.
3. In colocated repos (`.jj/` + `.git/`), use `jj` for mutations and keep `git` usage read-only unless the user explicitly requests otherwise.

## Use Agent-Safe Command Patterns

1. Pass a message flag instead of opening an editor:
   - `jj describe -m "message"`
   - `jj commit -m "message"`
   - `jj squash -m "message"`
2. Avoid commands that require interactive UI flows when running non-interactively.
3. After history mutations (`squash`, `rebase`, `abandon`, `restore`), run `jj status` to verify state.

## Run the Core Daily Flow

1. Inspect current state:

```bash
jj status
jj log -n 10
jj show
jj diff
```

2. Start and describe work:

```bash
jj new
jj describe -m "Implement feature X"
```

3. Continue editing files. Jujutsu tracks working-copy changes automatically.

4. Navigate the change graph without revision IDs:

```bash
jj next -e       # edit child change directly
jj prev -e       # edit parent change directly
jj edit <change> # jump to a specific change for editing
```

Without `-e`, `jj next`/`jj prev` create a new working-copy commit on top of the target revision.

## Handle History Edits

Use these commands for most agent workflows:

```bash
jj absorb
jj squash
jj split <path>... -m "split: <summary>"    # non-interactive split by file paths
jj rebase -r <revision> -d <destination>
jj simplify-parents                         # remove redundant merge parents after rebase
jj duplicate <revision> --destination <target> # copy a change onto an explicit target (cherry-pick style)
jj abandon <revision>
jj restore <path>
```

In agent contexts, always pass both filesets and `-m` to `jj split`.

Use `jj evolog` to inspect how a change has evolved over time (useful for debugging history mutations).

Load [cheat-sheet.md](./references/cheat-sheet.md) when the user describes a Git command and needs a Jujutsu equivalent.

## Push and Sync with Remotes

1. Sync from remote:

```bash
jj git fetch
jj rebase -d main@origin
```

2. Push a single change quickly:

```bash
jj git push --change @-
```

3. Use bookmarks for explicit branch-like collaboration:

```bash
jj bookmark create feat/my-change -r @-
jj git push -b feat/my-change
```

Load [colocated-git.md](./references/colocated-git.md) for teams that use GitHub/GitLab while you work locally in Jujutsu.

## Resolve Conflicts

1. Inspect conflict state:

```bash
jj status
jj resolve --list          # list all conflicted files
```

2. In non-interactive agent contexts, edit conflicted files directly and remove conflict markers. In interactive contexts, use the configured merge tool:

```bash
jj resolve <file>          # opens external merge tool (interactive only)
```

3. Re-run `jj status` until no unresolved conflicts remain.

## Recover Safely

Use operation-level undo when something goes wrong:

```bash
jj undo
jj op log
jj op restore <operation-id>
```

Prefer these commands instead of attempting destructive resets.

## Use Workspaces for Parallel Work

Workspaces let you check out multiple changes simultaneously in separate directories (like `git worktree`):

```bash
jj workspace add ../my-feature     # create a new workspace
jj workspace list                  # list all workspaces
jj workspace forget <name>         # remove a workspace
```

Each workspace has its own working-copy commit but shares the full change graph and operation log.

## Inspect Files

```bash
jj diff                            # diff of working copy against parent
jj diff -r <revision>              # diff of a specific revision
jj file annotate <path>            # blame/annotate a file (added in 0.24)
jj file list                       # list tracked files
```

## Auto-Format Code

If a formatting tool is configured, apply it across revisions:

```bash
jj fix
```

## Handle Version Differences

1. Run `jj --version`.
2. If a command flag behaves differently, run `jj help <subcommand>`.
3. Prefer command forms documented in [sources.md](./references/sources.md) when uncertain.