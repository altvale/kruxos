# Filesystem Capabilities

Read, write, search, copy, move, delete, watch, and restore files and directories. **12 capabilities in v0.0.1.**

## Overview

| Capability | Permission | Purpose |
|------------|:----------:|---------|
| [`filesystem.read`](#filesystemread) | 🟢 Autonomous | Reads the contents of a file at the specified path and returns the content with metadata. |
| [`filesystem.write`](#filesystemwrite) | 🔵 Notify | Writes content to a file at the specified path. |
| [`filesystem.edit`](#filesystemedit) | 🔵 Notify | Find-and-replace in existing files (literal or regex) with an expected-match-count guard; one file or many (glob); returns per-file counts + a unified diff. |
| [`filesystem.list`](#filesystemlist) | 🟢 Autonomous | Lists the contents of a directory, returning file and subdirectory entries with metadata. |
| [`filesystem.delete`](#filesystemdelete) | 🟡 Approval Required | Soft-deletes a file or directory into per-principal trash (168 h User / 24 h Agent retention). |
| [`filesystem.restore`](#filesystemrestore) | 🔵 Notify | Restores a soft-deleted file from trash back to its original path using sidecar metadata. |
| [`filesystem.stat`](#filesystemstat) | 🟢 Autonomous | Returns metadata about a file or directory without reading its contents. |
| [`filesystem.search`](#filesystemsearch) | 🟢 Autonomous | Recursively searches for files by glob/mtime/size — or, with `aggregate`, rolls up an inventory (count/size by extension, dir, or mtime day) and finds duplicate files (`include_hash` + `dedupe`) in one call. |
| [`filesystem.search_content`](#filesystemsearch_content) | 🔵 Notify | Searches file *contents* for a pattern (ranked, with context) — or counts them: `total_only` (just the total) and `count_by` (a histogram by a regex capture group). |
| [`filesystem.copy`](#filesystemcopy) | 🔵 Notify | Copies a file from one path to another. |
| [`filesystem.move`](#filesystemmove) | 🔵 Notify | Moves (renames) a file from one path to another. |
| [`filesystem.watch`](#filesystemwatch) | 🟢 Autonomous | Subscribes to filesystem change events in a directory. |
| [`filesystem.append`](#filesystemappend) | 🔵 Notify | Appends content to the end of an existing file. |

!!! info "Detailed schemas"
    Every capability's full input/output schema is emitted via MCP `tools/list` / JSON-RPC `capabilities.list`; the YAML source of truth lives at [`definitions/filesystem.yaml`](https://github.com/altvale/kruxos/blob/main/definitions/filesystem.yaml). The per-capability sections below cover the common surfaces; the newer aggregation modes (`filesystem.edit`, `search_content`'s `count_by`/`total_only`, and `search`'s `aggregate`/`include_hash`) are summarised here with the complete field lists in the YAML.

## `filesystem.read`

**Permission:** 🟢 Autonomous · **Version:** 1.0

> Reads the contents of a file at the specified path and returns the content with metadata.

### When to use

Use filesystem.read to retrieve file contents for analysis or processing.
Use filesystem.stat instead if you only need metadata without content.
Use filesystem.search to find files before reading them.

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `path` | `FilesystemPath` | Yes | — | Absolute path to the file. Must be within the agent's readable scope. Use agent.session to discover workspace boundaries. |
| `encoding` | `String` | No | `utf-8` | Character encoding for text files. Use 'binary' for non-text files to receive base64-encoded content. |
| `offset` | `Integer` | No | — | Byte offset to start reading from. Use with limit for paginated reads of large files. |
| `limit` | `Integer` | No | — | Maximum bytes to read. Default reads entire file. Use with offset for large files. |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `content` | `String` | File content as string (text) or base64 (binary). Compare length with size to detect truncation. |
| `size` | `Integer` | Total file size in bytes, regardless of offset/limit. |
| `modified` | `DateTime` | Last modification timestamp in ISO 8601 format. |
| `checksum` | `SHA256` | SHA-256 hash of the full file content. Use to verify integrity across operations. |
| `truncated` | `Boolean` | True if content was truncated due to limit parameter. Read remaining with offset. |

### Common patterns

**Read and verify file integrity**

1. `filesystem.read(path=...) to get content and checksum`
2. `Compare checksum with expected value`

**Read large file in chunks**

1. `filesystem.stat(path=...) to get total size`
2. `filesystem.read(path=..., offset=0, limit=1048576) for first 1MB`
3. `Continue with increasing offset until complete`

### Errors

**`PathOutOfScope`** — The path is outside the agent's readable scope.

- **check_scope**: Call agent.session to see readable directories.
- **request_access**: Request expanded access via alerts.send to supervisor.

**`FileNotFound`** — No file exists at the specified path.

- **search**: Use filesystem.search to find files matching a pattern.
- **list**: Use filesystem.list to see directory contents.

**`EncodingError`** — File content cannot be decoded with the specified encoding.

- **try_binary**: Retry with encoding='binary' to get base64-encoded raw bytes.

**Tags:** `filesystem` `read` `safe`

---

## `filesystem.write`

**Permission:** 🔵 Notify · **Version:** 1.0

> Writes content to a file at the specified path. Creates the file if it does not exist, overwrites if it does. Uses atomic write (temp file + rename) to prevent corruption.

### When to use

Use filesystem.write to create or update file contents.
Use filesystem.append if you want to add to an existing file without replacing it.
Use filesystem.copy to duplicate an existing file.

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `path` | `FilesystemPath` | Yes | — | Absolute path to write to. Must be within the agent's writable scope. Parent directories must exist. |
| `content` | `String` | Yes | — | Content to write. For binary data, provide base64-encoded content and set encoding to 'binary'. |
| `encoding` | `String` | No | `utf-8` | Character encoding. Use 'binary' to decode base64 content before writing. |
| `create_parents` | `Boolean` | No | `False` | If true, create parent directories that do not exist. |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `path` | `FilesystemPath` | The absolute path where the file was written. |
| `size` | `Integer` | Number of bytes written. |
| `checksum` | `SHA256` | SHA-256 hash of the written content. |

### Side effects

- Creates or overwrites a file on disk. *(reversible)*

### Common patterns

**Write a config file safely**

1. `filesystem.write(path='/app/config.json', content=...) — atomic write prevents partial writes`
2. `filesystem.read(path='/app/config.json') to verify content`

**Create a file with parent directories**

1. `filesystem.write(path='/data/reports/2024/q1.csv', content=..., create_parents=true)`

### Errors

**`PathOutOfScope`** — The path is outside the agent's writable scope.

- **check_scope**: Call agent.session to see writable directories.
- **request_access**: Request expanded access via alerts.send to supervisor.

**`ParentNotFound`** — A parent directory in the path does not exist.

- **create_parents**: Retry with create_parents=true to auto-create missing directories.

**`DiskFull`** — Not enough disk space to write the file.

- **check_quota**: Use system.disk_usage to check available space.

**Tags:** `filesystem` `write` `modifying`

---

## `filesystem.edit`

**Permission:** 🔵 Notify · **Version:** 1.0

> Find-and-replace inside existing files — literal or regex — with a server-side expected-match-count guard, returning per-file counts plus a unified diff of only the changed hunks.

Replaces the read-whole-file → reproduce-it-in-context → write-it-back loop (which sends the file through the context window twice) with a single call that carries only the find/replace strings up and only the changed hunks down.

### When to use

- Modify text that **already exists** — rename a symbol, bump a version, fix a config value, swap an import.
- You provide what to match (`pattern`), what to replace it with (`replace`), and how many matches you **expect** (`expected_replacements`); KruxOS verifies the count and **fails closed — nothing is written — if reality disagrees**, so a too-broad pattern can't silently corrupt the file.
- Use `filesystem.write` instead to create a file or replace its entire contents; `filesystem.append` to add to the end.

### Key inputs

| Input | Notes |
|-------|-------|
| `path` | The single file to edit, or — with `include_glob` — the directory root to edit across. |
| `include_glob` | Edit many files: a glob under `path` (e.g. `**/*.toml`). Default exclusions + `.gitignore` are honoured unless `include_all=true`. |
| `pattern` / `replace` | Literal by default; a regex with `${1}` backreferences when `regex=true`. |
| `expected_replacements` **xor** `replace_all` | Set exactly one. The count guard fails closed on mismatch; `replace_all` skips it. |
| `dry_run` | Preview the diff + per-file counts without writing. |

### Outputs

`applied` (`full`/`preview`/`partial`), `total_replacements`, `files_changed`, `per_file[]` (changed files only), `skipped[]` (binary/too-large), and `diff` — a unified diff that **is** the change record (no follow-up read needed to confirm). Full schema in the YAML.

### Example

```text
# preview a version bump across a tree, then commit
filesystem.edit(path='.', include_glob='**/*.toml', pattern='0.0.2', replace='0.0.3', dry_run=true)
filesystem.edit(path='.', include_glob='**/*.toml', pattern='0.0.2', replace='0.0.3', replace_all=true)
```

**Tags:** `filesystem` `edit` `write` `modifying`

---

## `filesystem.search_content`

**Permission:** 🔵 Notify · **Version:** 1.1

> Search file *contents* for a pattern in one call: ranked, deduplicated matches with smart context — **or** an aggregate that returns the count instead of streaming the matches back.

Pick the cheapest mode for your question (precedence: `total_only` > `count_by` > `files_only` > default):

| Mode | Returns |
|------|---------|
| `total_only=true` | Just the integer match total — "are there any? how many?" |
| `count_by='<group>'` | A histogram bucketed by a regex capture group (name, or 1-based index; **requires `regex=true`**). Log triage: `pattern='\b(ERROR\|WARN\|INFO)\b', regex=true, count_by='1'` → `{"ERROR":12,"INFO":900,"WARN":40}`. |
| `files_only=true` | One `{path, match_count}` row per matching file. |
| (default) | Ranked results with an `enclosing_snippet` so you usually need no follow-up read. |

Aggregate modes short-circuit the ranking/snippet work, so they are strictly cheaper than paging the default results and tallying by hand. `count_by`/`total_only` count occurrences; the default/`files_only` mode counts matching lines. Full schema in the YAML.

### Example

```text
# how many ERROR vs WARN vs INFO across the logs, in one call
filesystem.search_content(path='logs/', pattern='(ERROR|WARN|INFO)', regex=true, count_by='1')
```

**Tags:** `filesystem` `search` `read` `code` `aggregate`

---

## `filesystem.list`

**Permission:** 🟢 Autonomous · **Version:** 1.0

> Lists the contents of a directory, returning file and subdirectory entries with metadata.

### When to use

Use filesystem.list to see what files and directories exist at a path.
Use filesystem.search if you need to find files matching a pattern recursively.
Use filesystem.stat for detailed metadata about a single file.

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `path` | `FilesystemPath` | Yes | — | Absolute path to the directory. Must be within the agent's readable scope. |
| `include_hidden` | `Boolean` | No | `False` | If true, include files and directories starting with a dot. |
| `sort_by` | `String` | No | `name` | Sort order: 'name', 'size', 'modified'. Prefix with '-' for descending (e.g. '-modified'). |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `entries` | `Array` | Array of directory entries, each with name, type ('file'\|'directory'\|'symlink'), size, and modified timestamp. |
| `total` | `Integer` | Total number of entries returned. |
| `path` | `FilesystemPath` | The absolute path that was listed. |

### Common patterns

**Find recently modified files**

1. `filesystem.list(path='/data', sort_by='-modified')`
2. `Examine the first entries for recent changes`

**List all files including hidden**

1. `filesystem.list(path='/home/agent', include_hidden=true)`

### Errors

**`PathOutOfScope`** — The path is outside the agent's readable scope.

- **check_scope**: Call agent.session to see readable directories.

**`DirectoryNotFound`** — No directory exists at the specified path.

- **search**: Use filesystem.search to find directories matching a pattern.

**`NotADirectory`** — The path exists but is a file, not a directory.

- **read**: Use filesystem.read to read the file instead.

**Tags:** `filesystem` `read` `safe` `directory`

---

## `filesystem.delete`

**Permission:** 🟡 Approval Required · **Version:** 1.0

> Deletes a file or empty directory. Uses soft-delete — the item is moved to a trash location and can be recovered within 24 hours.

### When to use

Use filesystem.delete to remove files you no longer need.
The file is soft-deleted (moved to trash) and recoverable for 24 hours.
Use filesystem.list to verify the file exists before deleting.

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `path` | `FilesystemPath` | Yes | — | Absolute path to the file or empty directory to delete. |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `deleted_path` | `FilesystemPath` | The path that was deleted. |
| `trash_path` | `FilesystemPath` | Where the file was moved in the trash. Can be used to recover it. |
| `recoverable_until` | `DateTime` | ISO 8601 timestamp after which the file will be permanently deleted. |

### Side effects

- Moves the file to trash (soft-delete). Permanently removed after 24 hours. *(reversible)*

### Common patterns

**Delete and verify**

1. `filesystem.delete(path=...) — file moves to trash`
2. `filesystem.list(path=parent_dir) to confirm file is gone`

### Errors

**`PathOutOfScope`** — The path is outside the agent's writable scope.

- **check_scope**: Call agent.session to see writable directories.

**`FileNotFound`** — No file or directory exists at the specified path.

- **list**: Use filesystem.list to see what exists in the parent directory.

**`DirectoryNotEmpty`** — The path is a directory that contains files. Only empty directories can be deleted.

- **list_contents**: Use filesystem.list to see directory contents, then delete files individually.

**Tags:** `filesystem` `write` `destructive`

---

## `filesystem.stat`

**Permission:** 🟢 Autonomous · **Version:** 1.0

> Returns metadata about a file or directory without reading its contents. Faster than filesystem.read for metadata-only checks.

### When to use

Use filesystem.stat to check file existence, size, permissions, or modification time.
Use filesystem.read if you also need the file contents.
Use filesystem.list to get metadata for all entries in a directory.

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `path` | `FilesystemPath` | Yes | — | Absolute path to the file or directory. |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `exists` | `Boolean` | Whether the path exists. |
| `type` | `String` | Entry type: 'file', 'directory', or 'symlink'. |
| `size` | `Integer` | Size in bytes (0 for directories). |
| `modified` | `DateTime` | Last modification timestamp in ISO 8601. |
| `created` | `DateTime` | Creation timestamp in ISO 8601 (may not be available on all filesystems). |
| `permissions` | `String` | Unix permission string (e.g. 'rwxr-xr-x'). |
| `owner` | `String` | File owner username. |

### Common patterns

**Check if file exists before reading**

1. `filesystem.stat(path=...) to check exists and type`
2. `If exists and type is 'file', proceed with filesystem.read`

**Check file size before downloading**

1. `filesystem.stat(path=...) to get size`
2. `Compare size with expected to decide whether to re-download`

### Errors

**`PathOutOfScope`** — The path is outside the agent's readable scope.

- **check_scope**: Call agent.session to see readable directories.

**Tags:** `filesystem` `read` `safe` `metadata`

---

## `filesystem.search`

**Permission:** 🟢 Autonomous · **Version:** 1.0

> Recursively searches for files matching a glob pattern within a directory tree. Returns matching file paths with metadata.

### When to use

Use filesystem.search when you need to find files by name pattern.
Use filesystem.list if you just want to see the contents of a single directory.
Use filesystem.read to get the content of a file once you find it.

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `path` | `FilesystemPath` | Yes | — | Root directory to start searching from. Must be within the agent's readable scope. |
| `pattern` | `GlobPattern` | Yes | — | Glob pattern to match against file names (e.g. '*.json', '**/*.py', 'config.*'). |
| `max_results` | `Integer` | No | `100` | Maximum number of results to return. Use to prevent overwhelming output on large directory trees. |
| `include_hidden` | `Boolean` | No | `False` | If true, search inside hidden directories and match hidden files. |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `matches` | `Array` | Array of matching file paths with size and modified timestamp. |
| `total_matches` | `Integer` | Total number of matches found (may exceed max_results). |
| `truncated` | `Boolean` | True if total_matches exceeds max_results and results were truncated. |

### Common patterns

**Find all Python files in a project**

1. `filesystem.search(path='/workspace', pattern='**/*.py')`

**Find config files**

1. `filesystem.search(path='/etc', pattern='*.conf')`
2. `filesystem.read(path=...) for each result of interest`

### Errors

**`PathOutOfScope`** — The search root is outside the agent's readable scope.

- **check_scope**: Call agent.session to see readable directories.

**`DirectoryNotFound`** — The search root directory does not exist.

- **list**: Use filesystem.list on the parent directory to find available directories.

**`InvalidPattern`** — The glob pattern is syntactically invalid.

- **simplify**: Use a simpler pattern. Supported syntax: * (any), ** (recursive), ? (single char), [abc] (character class).

### Inventory & dedupe (v1.2)

Pass `aggregate` to roll the matched files up server-side instead of paging the per-file list and summing yourself — the per-file `matches` array is omitted and a compact table is returned:

- `aggregate.group_by` — `extension` (lowercased; no-extension/dotfile → `<none>`), `dir` (top-level subdirectory under `path`; a file directly under it → `.`), `mtime-bucket` (calendar day, UTC), or `none` (a grand total).
- `aggregate.metrics` — any of `count`, `total_size` (raw bytes). `sort_by` / `max_groups` order and bound the rows; an overflowing tail folds into an `<other>` row (`groups_truncated`).

```text
# total size by extension under /data, in one call
filesystem.search(path='/data', pattern='**/*', aggregate={group_by:'extension'})
```

Pass `include_hash=true` with `aggregate.dedupe=true` to get **duplicate-file groups** directly — files sharing an identical SHA-256, sorted biggest-reclaim-first — so you never compare hashes by hand. Hashing reads file bodies, so secret/policy-protected files are counted in the size totals but **never hashed**; files over `max_hash_bytes` (10 MiB default, 256 MiB max) are flagged in `large_files_skipped` rather than silently missed.

```text
# find duplicate photos to reclaim space
filesystem.search(path='/photos', pattern='**/*.jpg', include_hash=true, aggregate={dedupe:true})
```

**Tags:** `filesystem` `read` `safe` `search` `aggregate` `inventory` `dedupe`

---

## `filesystem.copy`

**Permission:** 🔵 Notify · **Version:** 1.0

> Copies a file from one path to another. Does not copy directories — use filesystem.list + filesystem.copy in a loop for that.

### When to use

Use filesystem.copy to duplicate a file.
Use filesystem.write if you want to create a new file with different content.
Use filesystem.move to relocate a file without copying.

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `source` | `FilesystemPath` | Yes | — | Absolute path to the file to copy. Must exist and be within readable scope. |
| `destination` | `FilesystemPath` | Yes | — | Absolute path for the copy. Must be within writable scope. Parent directories must exist. |
| `overwrite` | `Boolean` | No | `False` | If true, overwrite an existing file at the destination. If false and destination exists, returns an error. |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `source` | `FilesystemPath` | The source path that was copied from. |
| `destination` | `FilesystemPath` | The destination path of the new copy. |
| `size` | `Integer` | Size of the copied file in bytes. |
| `checksum` | `SHA256` | SHA-256 hash of the copied file. Compare with source checksum to verify. |

### Side effects

- Creates a new file at the destination path. *(reversible)*

### Common patterns

**Create a backup before modifying**

1. `filesystem.copy(source='/app/config.json', destination='/app/config.json.bak')`
2. `filesystem.write(path='/app/config.json', content=new_content)`

### Errors

**`PathOutOfScope`** — The source or destination is outside the agent's allowed scope.

- **check_scope**: Call agent.session to see readable and writable directories.

**`FileNotFound`** — The source file does not exist.

- **search**: Use filesystem.search to find the file.

**`DestinationExists`** — A file already exists at the destination and overwrite is false.

- **overwrite**: Retry with overwrite=true to replace the existing file.

**Tags:** `filesystem` `write` `modifying`

---

## `filesystem.move`

**Permission:** 🔵 Notify · **Version:** 1.0

> Moves (renames) a file from one path to another. The source file is removed after the move.

### When to use

Use filesystem.move to rename or relocate a file.
Use filesystem.copy if you want to keep the original.
Use filesystem.delete to remove a file without moving it.

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `source` | `FilesystemPath` | Yes | — | Absolute path to the file to move. Must exist and be within writable scope. |
| `destination` | `FilesystemPath` | Yes | — | Absolute path for the moved file. Must be within writable scope. |
| `overwrite` | `Boolean` | No | `False` | If true, overwrite an existing file at the destination. |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `source` | `FilesystemPath` | The original path (no longer exists). |
| `destination` | `FilesystemPath` | The new path where the file now resides. |
| `size` | `Integer` | Size of the moved file in bytes. |

### Side effects

- Removes the file from the source path and places it at the destination. *(reversible)*

### Common patterns

**Rename a file**

1. `filesystem.move(source='/data/report.txt', destination='/data/report-final.txt')`

### Errors

**`PathOutOfScope`** — The source or destination is outside the agent's writable scope.

- **check_scope**: Call agent.session to see writable directories.

**`FileNotFound`** — The source file does not exist.

- **search**: Use filesystem.search to find the file.

**`DestinationExists`** — A file already exists at the destination and overwrite is false.

- **overwrite**: Retry with overwrite=true to replace the existing file.

**Tags:** `filesystem` `write` `modifying`

---

## `filesystem.watch`

**Permission:** 🟢 Autonomous · **Version:** 1.0

> Subscribes to filesystem change events in a directory. Returns a watch_id that can be used to receive events when files are created, modified, or deleted.

### When to use

Use filesystem.watch to monitor a directory for changes in real time.
Use filesystem.list if you just want a one-time snapshot of directory contents.
Use filesystem.search if you want to find files matching a pattern right now, not watch for future changes.

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `path` | `FilesystemPath` | Yes | — | Absolute path to the directory to watch. Must be within the agent's readable scope. |
| `recursive` | `Boolean` | No | `False` | If true, watch all subdirectories recursively. Can be expensive on large directory trees. |
| `event_types` | `Array` | No | — | Filter for specific event types: 'create', 'modify', 'delete', 'rename'. Default watches all event types. |
| `pattern` | `GlobPattern` | No | — | Only report events for files matching this glob pattern (e.g. '*.log'). Default matches all files. |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `watch_id` | `String` | Unique identifier for this watch subscription. Use to cancel the watch or correlate events. |
| `path` | `FilesystemPath` | The directory being watched. |
| `recursive` | `Boolean` | Whether the watch is recursive. |

### Side effects

- Creates a filesystem watch using inotify. Consumes a kernel watch descriptor per directory. *(reversible)*

### Common patterns

**Watch for new log files**

1. `filesystem.watch(path='/var/log/app', pattern='*.log', event_types=['create'])`
2. `Poll session events or wait for push notification`
3. `filesystem.read the new file when event arrives`

**Watch and react to config changes**

1. `filesystem.watch(path='/etc/app', pattern='config.*')`
2. `On modify event, filesystem.read the changed config`
3. `Take action based on new config values`

### Errors

**`PathOutOfScope`** — The path is outside the agent's readable scope.

- **check_scope**: Call agent.session to see readable directories.

**`DirectoryNotFound`** — No directory exists at the specified path.

- **list**: Use filesystem.list on the parent directory to find available directories.

**`TooManyWatches`** — The kernel inotify watch limit has been reached.

- **reduce_scope**: Use a non-recursive watch or watch fewer directories.
- **cancel_existing**: Cancel an existing watch to free up a watch descriptor.

**Tags:** `filesystem` `read` `safe` `events` `realtime`

---

## `filesystem.append`

**Permission:** 🔵 Notify · **Version:** 1.0

> Appends content to the end of an existing file. Creates the file if it does not exist.

### When to use

Use filesystem.append to add data to a log file or growing dataset.
Use filesystem.write if you want to replace the entire file content.

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `path` | `FilesystemPath` | Yes | — | Absolute path to the file to append to. Must be within the agent's writable scope. |
| `content` | `String` | Yes | — | Content to append to the end of the file. |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `path` | `FilesystemPath` | The path of the appended file. |
| `new_size` | `Integer` | Total size of the file after appending. |

### Side effects

- Modifies the file by adding content at the end. *(not reversible)*

### Common patterns

**Append to a log file**

1. `filesystem.append(path='/data/agent.log', content='[2024-01-01] Operation completed\n')`

### Errors

**`PathOutOfScope`** — The path is outside the agent's writable scope.

- **check_scope**: Call agent.session to see writable directories.

**`DiskFull`** — Not enough disk space to append.

- **check_quota**: Use system.disk_usage to check available space.

**Tags:** `filesystem` `write` `modifying` `append`

---
