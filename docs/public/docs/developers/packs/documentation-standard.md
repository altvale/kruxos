# Capability Documentation Standard

Every capability must be documented for AI consumption. Agents read these definitions to decide which capability to use, how to call it, and how to handle errors. Write for machines, not humans.

## Required fields

Every capability definition in YAML must include these fields:

| Field | Purpose | Example |
|-------|---------|---------|
| `name` | Dotted identifier | `weather.current` |
| `version` | Semantic version | `"1.0"` |
| `purpose` | One-sentence description | "Returns the current weather..." |
| `when_to_use` | Multi-line guidance | When and when not to use this |
| `inputs` | Parameter definitions | See below |
| `outputs` | Return value definitions | See below |
| `side_effects` | What changes in the world | File writes, API calls, etc. |
| `common_patterns` | Usage recipes | Step-by-step guides |
| `errors` | What can go wrong | Type, description, recovery |
| `permission_tier` | Default permission level | `autonomous`, `notify`, etc. |
| `tags` | Categorisation tags | `["weather", "read", "safe"]` |

## Writing for AI agents

### Be concrete, not abstract

```yaml
# Bad — vague, metaphorical
purpose: "Opens a window into the filesystem."

# Good — concrete, literal
purpose: "Reads the content of a file at the given path and returns it as a string."
```

### Be specific about when to use and when NOT to use

```yaml
when_to_use: |
  Use filesystem.read to read the full content of a single file.
  For reading just the first N lines, use filesystem.read with the
  max_lines parameter. For reading multiple files, call filesystem.read
  once for each file — there is no batch read.
  Do NOT use filesystem.read for binary files — use filesystem.stat
  to check the file type first.
```

### Describe every input precisely

```yaml
inputs:
  - name: path
    type: String
    required: true
    description: >
      Absolute path to the file to read. Must start with / and must
      be within the agent's allowed directories (typically /workspace,
      /shared, /tmp). Relative paths are rejected with a ValidationError.
  - name: encoding
    type: String
    required: false
    default: "utf-8"
    description: >
      Character encoding to use when reading the file. Common values:
      'utf-8' (default), 'ascii', 'latin-1'. If the file contains bytes
      that are invalid for the encoding, a DecodingError is returned.
```

### Describe outputs completely

```yaml
outputs:
  - name: content
    type: String
    description: >
      The full file content as a string. For large files (>10 MB),
      consider using max_lines to read a portion.
  - name: size_bytes
    type: Integer
    description: "File size in bytes."
  - name: modified_at
    type: DateTime
    description: "ISO 8601 timestamp of last modification."
```

### Document every error with recovery actions

```yaml
errors:
  - type: FileNotFound
    description: "No file exists at the given path."
    recovery:
      - action: list_directory
        description: "Use filesystem.list on the parent directory to see available files."
      - action: check_path
        description: "Verify the path is absolute and correctly spelled."
  - type: PermissionDenied
    description: "The file exists but the agent's sandbox does not allow reading it."
    recovery:
      - action: check_policy
        description: "Use agent.policy to see which paths are accessible."
```

### Document side effects honestly

```yaml
side_effects:
  - description: "Creates or overwrites the file at the given path."
    reversible: true
  - description: "The previous file content is preserved in soft-delete for 24 hours."
    reversible: false
```

## Common patterns

Common patterns are step-by-step recipes. Write them as numbered steps with actual capability calls:

```yaml
common_patterns:
  - description: "Read a file, modify it, and write it back"
    steps:
      - "filesystem.read(path='/workspace/config.yaml') to get current content"
      - "Modify the content as needed"
      - "filesystem.write(path='/workspace/config.yaml', content=modified_content)"
  - description: "Check if a file exists before reading"
    steps:
      - "filesystem.stat(path='/workspace/data.csv') to check existence"
      - "If stat returns found=true, filesystem.read(path='/workspace/data.csv')"
```

## Tags

Use consistent tags across capabilities:

| Tag | Meaning |
|-----|---------|
| `read` | Reads data without modifying anything |
| `write` | Modifies data or state |
| `safe` | No side effects, always safe to call |
| `destructive` | Can cause permanent data loss |
| `buffered` | Write is delayed, can be cancelled |
| `cancellable` | Operation can be cancelled after initiation |
| `soft-delete` | Deletion preserves data for recovery window |

## Validation

Validate your definitions before publishing:

```bash
kruxos pack validate
```

This checks:

- All required fields are present
- Types are valid (`String`, `Integer`, `Boolean`, `Number`, `Object`, `Array`, `DateTime`)
- Permission tiers are valid (`autonomous`, `notify`, `approval_required`, `blocked`)
- Error types have at least one recovery action
- Common patterns have steps
- No duplicate capability names
