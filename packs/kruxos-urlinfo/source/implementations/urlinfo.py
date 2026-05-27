"""URL introspection pack implementations.

All network-touching capabilities reject `http://` inputs — v0.0.2's
KruxOS threat model treats plain-text traffic as out-of-scope. The
unit tests cover the parse + robots.txt-parser branches without any
network egress; the live HTTPS paths are exercised end-to-end by the
appliance after installation.
"""

import urllib.parse
import urllib.request
import urllib.robotparser

from kruxos.packs import capability, PackContext


def _require_https(url: str) -> None:
    if not url.startswith("https://"):
        raise ValueError(f"non_https_url: refusing {url!r}")


@capability("urlinfo.parse_host")
async def parse_host(ctx: PackContext, url: str) -> dict:
    """Parse a URL into scheme/host/port/path."""
    try:
        parsed = urllib.parse.urlsplit(url)
    except ValueError as e:
        raise ValueError(f"invalid_url: {e}") from e
    if not parsed.scheme or not parsed.hostname:
        raise ValueError(f"invalid_url: missing scheme/host in {url!r}")
    default_port = 443 if parsed.scheme == "https" else 80
    return {
        "scheme": parsed.scheme,
        "host": parsed.hostname.lower(),
        "port": parsed.port if parsed.port else default_port,
        "path": parsed.path or "",
    }


@capability("urlinfo.fetch_headers")
async def fetch_headers(
    ctx: PackContext, url: str, timeout_seconds: int = 10
) -> dict:
    """Issue an HTTPS HEAD and return status + headers."""
    _require_https(url)
    req = urllib.request.Request(url, method="HEAD")
    try:
        with urllib.request.urlopen(req, timeout=timeout_seconds) as resp:
            headers = {k.lower(): v for k, v in resp.headers.items()}
            return {
                "status": resp.status,
                "headers": headers,
                "final_url": resp.url,
            }
    except urllib.error.HTTPError as e:
        # An HTTP error response (4xx/5xx) is still a "headers result"
        # to the caller — they want to know the status.
        headers = {k.lower(): v for k, v in e.headers.items()} if e.headers else {}
        return {
            "status": e.code,
            "headers": headers,
            "final_url": e.url or url,
        }


@capability("urlinfo.robots_txt_check")
async def robots_txt_check(
    ctx: PackContext, url: str, user_agent: str = "*"
) -> dict:
    """Fetch and parse robots.txt, report whether the path is allowed."""
    _require_https(url)
    parsed = urllib.parse.urlsplit(url)
    robots_url = f"https://{parsed.hostname}/robots.txt"

    rp = urllib.robotparser.RobotFileParser()
    rp.set_url(robots_url)
    robots_status = 0
    try:
        # urllib.robotparser uses urllib.request internally; we fetch
        # explicitly so we can capture the HTTP status separately.
        with urllib.request.urlopen(robots_url, timeout=10) as resp:
            robots_status = resp.status
            body = resp.read().decode("utf-8", errors="replace")
            rp.parse(body.splitlines())
    except urllib.error.HTTPError as e:
        robots_status = e.code
        if e.code == 404:
            # No robots.txt → fully allowed per the de-facto standard.
            return {
                "allowed": True,
                "matched_rule": "",
                "robots_status": 404,
            }
        # Other 4xx/5xx is ambiguous — report as "robots_unreachable".
        raise RuntimeError(
            f"robots_unreachable: HTTP {e.code} fetching {robots_url}"
        ) from e

    path = parsed.path or "/"
    allowed = rp.can_fetch(user_agent, url)
    # The stdlib parser doesn't expose which rule matched; we report the
    # full path the parser was queried with as a hint.
    matched_rule = "" if allowed else f"{user_agent}:{path}"
    return {
        "allowed": allowed,
        "matched_rule": matched_rule,
        "robots_status": robots_status,
    }
