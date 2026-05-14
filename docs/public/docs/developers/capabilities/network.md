# Network Capabilities

HTTP requests, DNS lookups, file downloads, and port checks.

## Overview

| Capability | Permission | Purpose |
|------------|:----------:|---------|
| [`network.http_request`](#networkhttp_request) | 🔵 Notify | Makes an HTTP request to the specified URL and returns the response status, headers, and body. |
| [`network.dns_lookup`](#networkdns_lookup) | 🟢 Autonomous | Resolves a hostname to its IP addresses using DNS. |
| [`network.download`](#networkdownload) | 🔵 Notify | Downloads a file from a URL and saves it to the agent's workspace. |
| [`network.port_check`](#networkport_check) | 🟢 Autonomous | Checks whether a TCP port is open on a specified host by attempting a connection. |

## `network.http_request`

**Permission:** 🔵 Notify · **Version:** 1.0

> Makes an HTTP request to the specified URL and returns the response status, headers, and body.

### When to use

Use network.http_request to interact with web APIs, fetch data from URLs, or send data to external services.
Use network.download instead if you want to save a file to disk.
Use network.dns_lookup if you only need to resolve a hostname.
The request will be blocked if the target domain is not in the allowed domain list configured by the supervisor.

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `url` | `URL` | Yes | — | Full URL including scheme (http:// or https://). The domain must be in the supervisor-configured allowlist. |
| `method` | `String` | No | `GET` | HTTP method: GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS. |
| `headers` | `Object` | No | — | HTTP headers as key-value pairs. Example: {"Content-Type": "application/json", "Authorization": "Bearer token"}. |
| `body` | `String` | No | — | Request body as a string. For JSON payloads, serialize to string first. Ignored for GET and HEAD requests. |
| `timeout` | `Integer` | No | `30` | Request timeout in seconds. Maximum 300 seconds (5 minutes). |
| `follow_redirects` | `Boolean` | No | `True` | If true, automatically follow HTTP redirects (up to 10 hops). If false, return the redirect response directly. |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `status_code` | `Integer` | HTTP response status code (e.g. 200, 404, 500). |
| `headers` | `Object` | Response headers as key-value pairs. |
| `body` | `String` | Response body as a string. Binary responses are base64-encoded. |
| `duration_ms` | `Integer` | Total request duration in milliseconds including DNS, connection, and transfer. |
| `url` | `URL` | Final URL after any redirects. Same as input URL if no redirects occurred. |

### Side effects

- Sends an HTTP request to an external server. The server may perform actions based on the request (e.g. POST creates resources). *(not reversible)*

### Common patterns

**Fetch JSON from a REST API**

1. `network.http_request(url='https://api.example.com/data', headers={'Accept': 'application/json'})`
2. `Parse body as JSON to extract fields`

**POST data to an API**

1. `network.http_request(method='POST', url='https://api.example.com/items', headers={'Content-Type': 'application/json'}, body='{"name": "item"}')`
2. `Check status_code == 201 for success`

**Check if a URL is accessible**

1. `network.http_request(method='HEAD', url='https://example.com', timeout=5)`
2. `Check status_code for 200-299 range`

### Errors

**`DomainNotAllowed`** — The target domain is not in the supervisor-configured allowlist.

- **check_policy**: Use agent.policy to see which domains are allowed.
- **request_access**: Use alerts.send to request the supervisor add this domain to the allowlist.

**`InvalidUrl`** — The URL is malformed or missing a scheme.

- **fix_url**: Ensure the URL includes a scheme (https://) and is properly formatted.

**`RequestFailed`** — The HTTP request failed due to network error, timeout, or TLS error.

- **retry**: Retry the request. Transient network errors often resolve on retry.
- **increase_timeout**: If the error is a timeout, retry with a larger timeout value.

**`InvalidMethod`** — The specified HTTP method is not valid.

- **use_valid_method**: Use one of: GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS.

**Tags:** `network` `http` `egress`

---

## `network.dns_lookup`

**Permission:** 🟢 Autonomous · **Version:** 1.0

> Resolves a hostname to its IP addresses using DNS.

### When to use

Use network.dns_lookup to resolve hostnames before connecting, to verify DNS configuration,
or to check if a hostname exists. Use network.port_check to verify a service is reachable.

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `hostname` | `String` | Yes | — | The hostname to resolve (e.g. 'example.com'). Do not include a scheme or port. |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `addresses` | `Array` | Array of resolved IP address strings (IPv4 and IPv6). |
| `hostname` | `String` | The hostname that was resolved (echo of input). |

### Common patterns

**Verify a hostname resolves before making requests**

1. `network.dns_lookup(hostname='api.example.com')`
2. `Check addresses array is non-empty`
3. `Proceed with network.http_request`

### Errors

**`DnsResolutionFailed`** — The hostname could not be resolved. The domain may not exist or DNS may be unavailable.

- **check_hostname**: Verify the hostname is spelled correctly.
- **retry**: DNS failures can be transient. Retry after a short delay.

**Tags:** `network` `dns` `safe`

---

## `network.download`

**Permission:** 🔵 Notify · **Version:** 1.0

> Downloads a file from a URL and saves it to the agent's workspace. Returns the local path, file size, and checksum.

### When to use

Use network.download to save a remote file to disk for local processing.
Use network.http_request if you only need the response body in memory.
The destination path must be within the agent's workspace.

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `url` | `URL` | Yes | — | Full URL of the file to download. The domain must be in the supervisor-configured allowlist. |
| `path` | `FilesystemPath` | Yes | — | Destination path within the agent's workspace where the file will be saved. |
| `overwrite` | `Boolean` | No | `False` | If true, overwrite an existing file at the destination path. If false and the file exists, return an error. |
| `expected_checksum` | `SHA256` | No | — | Expected SHA-256 checksum to verify the downloaded file. If provided and does not match, the downloaded file is deleted and an error is returned. |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `path` | `FilesystemPath` | Absolute path where the file was saved. |
| `size` | `Integer` | Downloaded file size in bytes. |
| `checksum` | `SHA256` | SHA-256 checksum of the downloaded file. |
| `duration_ms` | `Integer` | Total download duration in milliseconds. |

### Side effects

- Creates or overwrites a file in the agent's workspace. *(reversible)*

### Common patterns

**Download and verify a file**

1. `network.download(url='https://example.com/data.tar.gz', path='/workspace/data.tar.gz', expected_checksum='abc123...')`
2. `On success, the file is verified and ready to use`

**Download then process a file**

1. `network.download(url='https://example.com/report.csv', path='/workspace/report.csv')`
2. `filesystem.read(path='/workspace/report.csv') to process the contents`

### Errors

**`DomainNotAllowed`** — The target domain is not in the supervisor-configured allowlist.

- **check_policy**: Use agent.policy to see which domains are allowed.

**`InvalidUrl`** — The URL is malformed or missing a scheme.

- **fix_url**: Ensure the URL includes a scheme (https://) and is properly formatted.

**`PathOutOfScope`** — The destination path is outside the agent's workspace.

- **check_scope**: Call agent.session to see accessible directories.

**`DestinationExists`** — A file already exists at the destination path.

- **overwrite**: Retry with overwrite=true to replace the existing file.

**`ChecksumMismatch`** — The downloaded file's SHA-256 checksum does not match the expected value. The file has been deleted.

- **retry**: Retry the download. Partial or corrupted transfers can happen.
- **verify_checksum**: Check that the expected_checksum value is correct.

**`DownloadFailed`** — The download failed due to a network error.

- **retry**: Retry the download. Transient network errors often resolve on retry.

**Tags:** `network` `download` `filesystem` `egress`

---

## `network.port_check`

**Permission:** 🟢 Autonomous · **Version:** 1.0

> Checks whether a TCP port is open on a specified host by attempting a connection.

### When to use

Use network.port_check to verify a service is running and reachable before attempting to connect.
Use network.dns_lookup if you only need to verify DNS resolution.

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `host` | `String` | Yes | — | Hostname or IP address to check. |
| `port` | `Integer` | Yes | — | TCP port number to check (1-65535). |
| `timeout` | `Integer` | No | `5` | Connection timeout in seconds. Maximum 30 seconds. |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `open` | `Boolean` | True if the port accepted a TCP connection within the timeout period. |
| `latency_ms` | `Integer` | Connection latency in milliseconds. Null if the port is closed. |
| `host` | `String` | The host that was checked (echo of input). |
| `port` | `Integer` | The port that was checked (echo of input). |

### Common patterns

**Check if a web server is running**

1. `network.port_check(host='localhost', port=8080)`
2. `If open==true, proceed with network.http_request`

**Wait for a service to start**

1. `network.port_check(host='db-host', port=5432, timeout=10)`
2. `If open==false, wait and retry`

### Errors

**`InvalidPort`** — The port number is outside the valid range (1-65535).

- **fix_port**: Use a port number between 1 and 65535.

**`ConnectionFailed`** — Could not establish a connection. The host may be unreachable.

- **check_host**: Use network.dns_lookup to verify the hostname resolves.
- **retry**: Retry after a short delay.

**Tags:** `network` `diagnostics` `safe`

---
