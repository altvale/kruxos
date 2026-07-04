# Git Capabilities

Clone, pull, push, commit, diff, log, branch, and working-tree state. **8 capabilities in v0.0.1.**

## Overview

| Capability | Permission | Purpose |
|------------|:----------:|---------|
| [`git.clone`](#gitclone) | 🔵 Notify | Clones a remote Git repository into the agent's workspace. |
| [`git.pull`](#gitpull) | 🔵 Notify | Fetches and merges changes from the remote repository into the local branch. |
| [`git.push`](#gitpush) | 🟡 Approval Required | Pushes local commits to the remote repository. |
| [`git.commit`](#gitcommit) | 🔵 Notify | Creates a Git commit with the specified files and message in the local repository. |
| [`git.diff`](#gitdiff) | 🟢 Autonomous | Shows the differences between the working directory, index, or commits in a Git repository. |
| [`git.log`](#gitlog) | 🟢 Autonomous | Returns the commit history of a Git repository, most recent first. |
| [`git.branch`](#gitbranch) | 🔵 Notify | Lists, creates, or switches branches in a Git repository. |
| [`git.status`](#gitstatus) | 🟢 Autonomous | Returns the current state of the working tree (branch, staged / unstaged / untracked / conflicted lists, upstream ahead-behind, repository state). |

!!! info "git.status schema"
    The full input/output schema for `git.status` is emitted via MCP `tools/list` / JSON-RPC `capabilities.list`; the YAML source of truth lives at [`definitions/git.yaml`](https://github.com/altvale/kruxos/blob/main/definitions/git.yaml).

## `git.clone`

**Permission:** 🔵 Notify · **Version:** 1.0

> Clones a remote Git repository into the agent's workspace.

### When to use

Use git.clone to download a repository for the first time.
Use git.pull if the repository already exists locally and you need to update it.
The destination path must be within the agent's workspace.

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `url` | `URL` | Yes | — | Remote repository URL (HTTPS or SSH). Example: 'https://github.com/user/repo.git'. |
| `path` | `FilesystemPath` | Yes | — | Destination directory within the agent's workspace. Will be created if it does not exist. |
| `branch` | `String` | No | — | Branch to check out after cloning. Defaults to the repository's default branch. |
| `depth` | `Integer` | No | — | Create a shallow clone with this many commits of history. Omit for full clone. |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `path` | `FilesystemPath` | Absolute path to the cloned repository. |
| `default_branch` | `String` | Name of the repository's default branch (e.g. 'main'). |
| `head_commit` | `String` | SHA hash of the HEAD commit after cloning. |

### Side effects

- Creates a directory tree containing the cloned repository in the agent's workspace. *(reversible)*

### Common patterns

**Clone and inspect a repository**

1. `git.clone(url='https://github.com/user/repo.git', path='/workspace/repo')`
2. `git.log(path='/workspace/repo', limit=5) to see recent commits`
3. `filesystem.list(path='/workspace/repo') to see the file structure`

**Clone a specific branch**

1. `git.clone(url='https://github.com/user/repo.git', path='/workspace/repo', branch='develop')`

### Errors

**`PathOutOfScope`** — The destination path is outside the agent's workspace.

- **check_scope**: Call agent.session to see accessible directories.

**`CloneFailed`** — The clone operation failed. The remote URL may be invalid, the repository may not exist, or authentication may have failed.

- **check_url**: Verify the repository URL is correct and accessible.
- **check_auth**: Use secrets.list to check if SSH keys or credentials are configured.

**`DestinationExists`** — The destination directory already exists and is not empty.

- **use_pull**: If the repo is already cloned, use git.pull to update it.
- **choose_path**: Choose a different destination path.

**Tags:** `git` `vcs` `egress`

---

## `git.pull`

**Permission:** 🔵 Notify · **Version:** 1.0

> Fetches and merges changes from the remote repository into the local branch.

### When to use

Use git.pull to update an existing local repository with remote changes.
Use git.clone if the repository does not exist locally yet.
Use git.diff after pulling to inspect what changed.

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `path` | `FilesystemPath` | Yes | — | Path to the local Git repository root directory. |
| `remote` | `String` | No | `origin` | Name of the remote to pull from. |
| `branch` | `String` | No | — | Remote branch to pull. Defaults to the current branch's upstream. |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `new_commits` | `Integer` | Number of new commits received from the remote. |
| `files_changed` | `Integer` | Number of files changed by the merge. |
| `head_commit` | `String` | SHA hash of the HEAD commit after pulling. |
| `up_to_date` | `Boolean` | True if the local branch was already up to date with the remote. |

### Side effects

- Modifies files in the repository working directory to reflect remote changes. *(reversible)*

### Common patterns

**Update repo and check what changed**

1. `git.pull(path='/workspace/repo')`
2. `If up_to_date==false, use git.diff to inspect changes`

### Errors

**`PathOutOfScope`** — The repository path is outside the agent's workspace.

- **check_scope**: Call agent.session to see accessible directories.

**`NotARepository`** — The specified path is not a Git repository.

- **clone_first**: Use git.clone to clone the repository first.

**`MergeConflict`** — The pull resulted in merge conflicts that need manual resolution.

- **check_diff**: Use git.diff to see conflicting files.

**`PullFailed`** — The pull operation failed due to a network or authentication error.

- **retry**: Retry the operation.
- **check_auth**: Use secrets.list to check if credentials are configured.

**Tags:** `git` `vcs` `egress`

---

## `git.push`

**Permission:** 🟡 Approval Required · **Version:** 1.0

> Pushes local commits to the remote repository.

### When to use

Use git.push after git.commit to send your changes to the remote.
Use git.pull first if the remote has new commits, to avoid push rejection.

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `path` | `FilesystemPath` | Yes | — | Path to the local Git repository root directory. |
| `remote` | `String` | No | `origin` | Name of the remote to push to. |
| `branch` | `String` | No | — | Branch to push. Defaults to the current branch. |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `pushed_refs` | `Array` | Array of ref strings that were pushed (e.g. ['refs/heads/main']). |
| `head_commit` | `String` | SHA hash of the HEAD commit that was pushed. |

### Side effects

- Sends commits to the remote repository. Other collaborators and CI systems will see the changes. *(not reversible)*

### Common patterns

**Commit and push changes**

1. `git.commit(path='/workspace/repo', message='Add feature', files=['src/feature.rs'])`
2. `git.push(path='/workspace/repo')`

### Errors

**`PathOutOfScope`** — The repository path is outside the agent's workspace.

- **check_scope**: Call agent.session to see accessible directories.

**`NotARepository`** — The specified path is not a Git repository.

- **clone_first**: Use git.clone to clone the repository first.

**`PushRejected`** — The remote rejected the push. This usually means the remote has new commits.

- **pull_first**: Use git.pull to fetch and merge remote changes, then push again.

**`PushFailed`** — The push operation failed due to a network or authentication error.

- **retry**: Retry the operation.
- **check_auth**: Use secrets.list to check if credentials are configured.

**Tags:** `git` `vcs` `egress` `destructive`

---

## `git.commit`

**Permission:** 🔵 Notify · **Version:** 1.0

> Creates a Git commit with the specified files and message in the local repository.

### When to use

Use git.commit to save a set of changes as a commit in the local repository.
Use git.push afterwards to send the commit to the remote.
Use git.diff before committing to review what will be committed.

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `path` | `FilesystemPath` | Yes | — | Path to the local Git repository root directory. |
| `message` | `String` | Yes | — | Commit message describing the changes. |
| `files` | `Array` | No | — | Array of file paths (relative to repo root) to stage and commit. If omitted, commits all modified and new files. |
| `author_name` | `String` | No | — | Author name for the commit. Defaults to the repository's configured user.name. |
| `author_email` | `String` | No | — | Author email for the commit. Defaults to the repository's configured user.email. |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `commit_hash` | `String` | SHA hash of the new commit. |
| `files_committed` | `Integer` | Number of files included in the commit. |
| `branch` | `String` | Name of the branch the commit was created on. |

### Side effects

- Creates a new commit object in the local Git repository. Does not modify the remote. *(reversible)*

### Common patterns

**Stage specific files and commit**

1. `git.diff(path='/workspace/repo') to review changes`
2. `git.commit(path='/workspace/repo', message='Fix bug in parser', files=['src/parser.rs', 'tests/parser_test.rs'])`

**Commit all changes**

1. `git.commit(path='/workspace/repo', message='Update dependencies')`
2. `git.push(path='/workspace/repo')`

### Errors

**`PathOutOfScope`** — The repository path is outside the agent's workspace.

- **check_scope**: Call agent.session to see accessible directories.

**`NotARepository`** — The specified path is not a Git repository.

- **clone_first**: Use git.clone to clone the repository first.

**`NothingToCommit`** — There are no staged or modified files to commit.

- **check_status**: Use git.diff to see if there are any changes.

**`CommitFailed`** — The commit operation failed.

- **check_config**: Ensure user.name and user.email are configured, or provide author_name and author_email.

**Tags:** `git` `vcs` `write`

---

## `git.diff`

**Permission:** 🟢 Autonomous · **Version:** 1.0

> Shows the differences between the working directory, index, or commits in a Git repository.

### When to use

Use git.diff to review uncommitted changes before committing.
Use git.log to see commit history instead of individual diffs.
Use git.diff with a commit hash to compare against a specific commit.

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `path` | `FilesystemPath` | Yes | — | Path to the local Git repository root directory. |
| `cached` | `Boolean` | No | `False` | If true, show only staged changes (index vs HEAD). If false, show unstaged changes (working directory vs index). |
| `commit` | `String` | No | — | Compare working directory against this commit hash instead of the index. |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `changes` | `Array` | Array of change objects, each with: file (path), status ('added', 'modified', 'deleted', 'renamed'), additions (line count), deletions (line count). |
| `total_additions` | `Integer` | Total number of lines added across all files. |
| `total_deletions` | `Integer` | Total number of lines deleted across all files. |
| `files_changed` | `Integer` | Number of files with changes. |

### Common patterns

**Review changes before committing**

1. `git.diff(path='/workspace/repo') to see unstaged changes`
2. `git.diff(path='/workspace/repo', cached=true) to see staged changes`
3. `git.commit(path='/workspace/repo', message='...') to commit`

### Errors

**`PathOutOfScope`** — The repository path is outside the agent's workspace.

- **check_scope**: Call agent.session to see accessible directories.

**`NotARepository`** — The specified path is not a Git repository.

- **clone_first**: Use git.clone to clone the repository first.

**`CommitNotFound`** — The specified commit hash does not exist in the repository.

- **check_log**: Use git.log to find valid commit hashes.

**Tags:** `git` `vcs` `safe` `read`

---

## `git.log`

**Permission:** 🟢 Autonomous · **Version:** 1.0

> Returns the commit history of a Git repository, most recent first.

### When to use

Use git.log to inspect commit history, find specific commits, or understand project evolution.
Use git.diff to see the actual content changes within a commit.

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `path` | `FilesystemPath` | Yes | — | Path to the local Git repository root directory. |
| `limit` | `Integer` | No | `20` | Maximum number of commits to return. Default 20, maximum 500. |
| `branch` | `String` | No | — | Branch to show history for. Defaults to the current branch. |
| `since` | `DateTime` | No | — | Only show commits after this timestamp (ISO 8601). |
| `author` | `String` | No | — | Filter commits by author name or email (substring match). |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `commits` | `Array` | Array of commit objects, each with: hash (full SHA), short_hash (7 chars), message, author_name, author_email, timestamp (ISO 8601). |
| `total_returned` | `Integer` | Number of commits returned. |
| `branch` | `String` | Branch name the log was generated from. |

### Common patterns

**View recent commits**

1. `git.log(path='/workspace/repo', limit=10)`

**Find commits by a specific author**

1. `git.log(path='/workspace/repo', author='alice', limit=50)`

### Errors

**`PathOutOfScope`** — The repository path is outside the agent's workspace.

- **check_scope**: Call agent.session to see accessible directories.

**`NotARepository`** — The specified path is not a Git repository.

- **clone_first**: Use git.clone to clone the repository first.

**`BranchNotFound`** — The specified branch does not exist in the repository.

- **list_branches**: Use git.branch(path=..., action='list') to see available branches.

**Tags:** `git` `vcs` `safe` `read`

---

## `git.branch`

**Permission:** 🔵 Notify · **Version:** 1.0

> Lists, creates, or switches branches in a Git repository.

### When to use

Use git.branch with action='list' to see all branches.
Use git.branch with action='create' to create a new branch.
Use git.branch with action='switch' to check out a different branch.
Use git.branch with action='delete' to remove a local branch.

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `path` | `FilesystemPath` | Yes | — | Path to the local Git repository root directory. |
| `action` | `String` | Yes | — | One of: 'list' (show branches), 'create' (new branch), 'switch' (checkout branch), 'delete' (remove branch). |
| `name` | `String` | No | — | Branch name for create, switch, or delete actions. Not required for list. |
| `start_point` | `String` | No | — | Commit hash or branch name to start the new branch from. Only used with action='create'. Defaults to HEAD. |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `branches` | `Array` | Array of branch objects, each with: name, is_current (boolean), is_remote (boolean). Returned for all actions. |
| `current_branch` | `String` | Name of the currently checked-out branch after the operation. |

### Side effects

- Creating or switching branches modifies the repository state. Switching branches changes files in the working directory. *(reversible)*

### Common patterns

**Create a feature branch and switch to it**

1. `git.branch(path='/workspace/repo', action='create', name='feature/new-thing')`
2. `git.branch(path='/workspace/repo', action='switch', name='feature/new-thing')`

**List all branches**

1. `git.branch(path='/workspace/repo', action='list')`

### Errors

**`PathOutOfScope`** — The repository path is outside the agent's workspace.

- **check_scope**: Call agent.session to see accessible directories.

**`NotARepository`** — The specified path is not a Git repository.

- **clone_first**: Use git.clone to clone the repository first.

**`BranchNotFound`** — The specified branch does not exist.

- **list_branches**: Use git.branch(path=..., action='list') to see available branches.

**`BranchAlreadyExists`** — A branch with that name already exists.

- **switch**: Use action='switch' to check out the existing branch.
- **choose_name**: Choose a different branch name.

**`InvalidAction`** — The action must be one of: list, create, switch, delete.

- **use_valid_action**: Use one of: list, create, switch, delete.

**Tags:** `git` `vcs` `branch`

---
