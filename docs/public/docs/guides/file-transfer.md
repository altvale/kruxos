# File Transfer

How to get a file from your workstation onto a KruxOS appliance — to install a
pack from a local tarball, drop in a custom cert / license JWT / config
payload, or seed a state file — without leaving the dashboard.

## Why this exists

The KruxOS appliance ships a deliberately minimal base image: no hypervisor
guest tools and no general-purpose shell access for agents, with the SSH server
**disabled by default**. That's good for the security posture, but it means
"just `scp` the file over" isn't available until you opt in. The first-party
file-transfer surfaces below give operators a supported path that lands files in
a known location with auditing and size limits — and once SSH is enabled,
`scp` / SFTP becomes a first-party path too (see the SSH section below).

## Surfaces

### 1. Dashboard `/uploads` page (recommended)

Open `https://<appliance>:7800/uploads`. The page gives you drag-and-drop, a
file picker, a list of what's already there, and per-file delete. Uploaded
files land at `/data/kruxos/uploads/<name>` and the page shows the full
appliance path so you can copy-paste it into other tools — for example, to
install a local pack, upload its tarball, extract it into a directory under
`/data/kruxos/uploads/`, and point `kruxos pack install` at that directory
(local installs take a pack *directory*, not a tarball).

The page is gated by your dashboard session; the upload itself is forwarded to
the gateway as the **User principal**, the same identity the rest of the
loopback User API uses.

### 2. User API `POST /api/user/files` (for scripts and tooling)

The same backing endpoint the dashboard uses, for shell scripts and
automation. The User API listens on the appliance loopback at `:7703`, so run
this from the appliance itself (or through an SSH tunnel you've set up):

```bash
# $KRUXOS_USER_TOKEN is a krx_user_* bearer — create one with
# `kruxos user-token create <label>` or copy it from the dashboard.
curl -X POST http://127.0.0.1:7703/api/user/files \
     -H "Authorization: Bearer $KRUXOS_USER_TOKEN" \
     -F "file=@./my-pack.tar.gz"
# 201 Created
# {"name":"my-pack.tar.gz","path":"/data/kruxos/uploads/my-pack.tar.gz","size_bytes":12345}
```

Behaviour:

| Aspect | Value |
|--------|-------|
| Backing path | `/data/kruxos/uploads/` |
| Per-upload size cap | 100 MiB (returns `413 Payload Too Large` if exceeded) |
| Filename rules | Path components are stripped; the basename must be ≤ 255 bytes, must not start with `.`, and may only contain `A–Z a–z 0–9 . _ - ( )` and spaces |
| Overwrite | Re-uploading an existing name returns `409 Conflict`; pass `?overwrite=true` to replace |
| Auth | `krx_user_*` bearer token (same as the rest of `/api/user/*`) |
| Audit | Emits `user.file.uploaded` / `user.file.deleted` events |

### 3. List and delete

```bash
# List uploads — returns [{name, size_bytes, modified}], sorted by name
curl http://127.0.0.1:7703/api/user/files \
     -H "Authorization: Bearer $KRUXOS_USER_TOKEN"

# Delete one upload — 204 on success, 404 if it doesn't exist
curl -X DELETE http://127.0.0.1:7703/api/user/files/my-pack.tar.gz \
     -H "Authorization: Bearer $KRUXOS_USER_TOKEN"
```

## Other transfer mechanisms (situational)

These are not first-party features and come with caveats. Use the upload
surface above unless your situation specifically calls for one of these.

### Docker bind-mount

When the appliance runs under Docker Compose, the persistent named volume is
mounted at `/data/kruxos`. You can substitute a bind-mount to expose a host
directory:

```yaml
# Excerpt from a docker-compose override
services:
  kruxos:
    volumes:
      - ./local/uploads:/data/kruxos/uploads:rw
```

Files dropped into the host's `./local/uploads/` then appear at
`/data/kruxos/uploads/` inside the container, where the file API sees them on
the next list call.

### VM hypervisor shared folder / USB

Hypervisor-specific. The appliance doesn't ship VirtualBox / VMware / virtiofs
guest tools (the rootfs is intentionally minimal), so shared folders only work
at the hypervisor level — e.g. a host directory exposed as an extra block
device that you mount manually inside the VM. USB passthrough is similar: the
hypervisor exposes the device, you mount it under `/mnt` and copy files into
`/data/kruxos/uploads/`.

### `kruxos pack install <name>` for published packs

For packs published to a registry, `kruxos pack install <name>` fetches and
installs by name — no local file transfer needed. The upload flow is for
unpublished or private packs (upload, extract to a directory, then
`kruxos pack install <directory>`).

### Console paste (debugging only)

The VM hypervisor console supports paste, so very small files (a config
snippet, a license JWT for offline activation) can be reconstructed with a
heredoc:

```bash
cat > /tmp/file.txt <<'EOF'
<pasted content>
EOF
```

Caveats: markdown indentation can break heredoc terminators; serial-console
flow-control drops characters on large pastes; not viable for anything over a
few KB.

### SSH (opt-in)

As of v0.0.3 the appliance bundles an OpenSSH server, but it is **opt-in and
disabled by default** — nothing listens and `tcp/22` stays firewalled until you
enable it from **Settings › System › SSH access**. Enabling requires at least
one authorized public key; KruxOS then starts the SSH service and opens the
firewall rule, and removes that rule again when you disable it.

#### Set up an SSH key

Because SSH is public-key only, you add a key before the service will start. If
you don't already have one, generate a key pair on your workstation:

```bash
ssh-keygen -t ed25519
```

Press Enter to accept the default path (`~/.ssh/id_ed25519`); the optional
passphrase it offers encrypts the private key at rest. This writes two files —
the private key `id_ed25519` (keep it secret) and the public key
`id_ed25519.pub` (the one you share).

Print the **public** key so you can copy it:

=== "macOS / Linux"

    ```bash
    cat ~/.ssh/id_ed25519.pub
    ```

=== "Windows (PowerShell)"

    ```powershell
    type $env:USERPROFILE\.ssh\id_ed25519.pub
    ```

It is a single line beginning with `ssh-ed25519 AAAA…`. Copy that whole line,
open **Settings › System › SSH access** in the dashboard, paste it into the SSH
card, and save. The card shows the key's `SHA256:…` fingerprint — confirm it
matches the fingerprint `ssh-keygen` printed when you created the key (or
re-derive it with `ssh-keygen -lf ~/.ssh/id_ed25519.pub`) so you know the right
key was pasted intact. Once the first key is saved, KruxOS starts the SSH
service and opens `tcp/22`.

!!! warning "Paste the public key, never the private one"
    Only the `.pub` file ever leaves your workstation. The private key (the
    `id_ed25519` file with no extension, and anything containing
    `BEGIN … PRIVATE KEY`) must never be pasted anywhere. Each authorized key
    is a **root credential in its own right**, independent of the vault
    passphrase: changing the appliance passphrase does **not** revoke keys, and
    removing a key does **not** change the passphrase. Add only keys you
    control, and remove a key as soon as the workstation holding it is
    decommissioned.

The posture is locked down: **root login, public-key only — password
authentication is never enabled**, so the appliance passphrase is never exposed
over the network. Host keys and your `authorized_keys` live on the data
partition and survive A/B updates. SFTP and `scp` run over the same connection,
so once SSH is on it is also a first-party file-transfer path:

```bash
# copy a local pack tarball onto the appliance over SSH
scp ./my-pack.tar.gz root@<appliance>:/data/kruxos/uploads/
```

Keep SSH on the LAN or behind a VPN — the management plane is not meant for the
public internet. **SSH stays on across reboot until you disable it**, so turn it
off from the same SSH card when you don't need it. Removing your last authorized
key automatically disables the service and closes `tcp/22` again.

## Security model

- The upload directory is **User-scoped**, not per-agent-scoped. Files land in
  a directory readable by any process running as the User principal (the
  gateway itself, `kruxos pack install`, and so on).
- Sandboxed agents and MCP clients do **not** get access to
  `/data/kruxos/uploads/` unless you explicitly mount it into the agent's
  sandbox — same governance as any other host directory.
- The audit events (`user.file.uploaded` / `user.file.deleted`) record
  filename and size, not contents. Watch the audit feed to catch surprise
  uploads.
- Filename sanitisation rejects path components, dotfiles, and characters that
  shells / HTTP / path resolvers interpret specially. A symlink swapped into
  the uploads directory is caught by canonicalising the parent before the file
  is opened.

## Next steps

- [Getting Started](../getting-started.md) — stand up the appliance and the
  dashboard first
- [Managing Agents](managing-agents.md) — control which agents can reach
  host paths
- [Monitoring](monitoring.md) — watch the audit feed for upload/delete events
