# Publishing Packs

Share your capability pack with the KruxOS community.

!!! warning "Publishing flow ships in v0.0.2"
    `kruxos pack validate / test / build / login / publish` and the GitHub-based publishing pipeline all ship in **v0.0.2**. v0.0.1 supports local-path installs only via `kruxos pack install <local-path>`. The rest of this page describes the v0.0.2 surface.

## Pre-publish checklist

```bash
# Validate everything
kruxos pack validate --strict

# Run tests
kruxos pack test

# Build the pack
kruxos pack build
```

The `build` command creates a distributable archive:

```
dist/
└── my-weather-pack-1.0.0.tar.gz
```

## Publishing to the registry

### 1. Create a registry account

```bash
kruxos pack login
```

This opens a browser for GitHub authentication. Your GitHub identity is your registry identity.

### 2. Publish

```bash
kruxos pack publish
```

Output:

```
Publishing my-weather-pack v1.0.0...
  ✓ Validation passed
  ✓ Tests passed (4/4)
  ✓ Built package (12 KB)
  ✓ Published to registry

Package URL: https://packs.kruxos.com/my-weather-pack
Install:     kruxos pack install my-weather-pack
```

### 3. Update an existing pack

Bump the version in `pack.yaml`, then publish again:

```bash
kruxos pack publish
```

The registry enforces semantic versioning — you cannot overwrite an existing version.

## Registry format

The v0.0.2 registry is a GitHub repository that acts as a package index (like Homebrew taps). Each pack is a directory in the registry containing:

```
registry/
├── my-weather-pack/
│   ├── metadata.yaml      # Pack info, author, versions
│   ├── 1.0.0.tar.gz       # Version 1.0.0 archive
│   └── 1.1.0.tar.gz       # Version 1.1.0 archive
└── another-pack/
    └── ...
```

### metadata.yaml

```yaml
name: my-weather-pack
author: github-username
description: "Weather data capabilities for KruxOS agents"
license: MIT
homepage: https://github.com/you/my-weather-pack
versions:
  - version: "1.0.0"
    published_at: "2026-03-15T10:00:00Z"
    sha256: "abc123..."
    capabilities: ["weather.current"]
  - version: "1.1.0"
    published_at: "2026-03-20T14:00:00Z"
    sha256: "def456..."
    capabilities: ["weather.current", "weather.forecast"]
```

## Installing published packs

```bash
# Install latest version
kruxos pack install my-weather-pack

# Install specific version
kruxos pack install my-weather-pack@1.0.0

# Update to latest
kruxos pack update my-weather-pack

# List installed packs
kruxos pack list

# Remove a pack
kruxos pack remove my-weather-pack
```

## Naming guidelines

- Use lowercase with hyphens: `my-weather-pack`, not `MyWeatherPack`
- Be descriptive: `slack-integration` not `slack`
- Avoid generic names: `csv-tools` not `tools`
- Capability names should use your pack name as a prefix: `weather.current`, `slack.send_message`

## Versioning

Follow [semantic versioning](https://semver.org/):

| Change | Version bump | Example |
|--------|-------------|---------|
| Bug fix | Patch | 1.0.0 → 1.0.1 |
| New capability | Minor | 1.0.0 → 1.1.0 |
| Breaking change | Major | 1.0.0 → 2.0.0 |

Breaking changes include:

- Removing a capability
- Changing a capability's required inputs
- Changing a capability's output structure
- Changing a capability's name
