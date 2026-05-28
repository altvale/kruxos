"""kruxos-hashutils — Capability implementations.

Each function implements one capability defined in definitions/*.yaml.
The function name matches the capability name with dots replaced by
underscores (`hash.sha256` -> `hash_sha256`).
"""

import hashlib
import uuid
from typing import Any


def hash_sha256(text: str) -> dict[str, Any]:
    """Return the SHA-256 hex digest of text."""
    encoded = text.encode("utf-8")
    return {
        "hex": hashlib.sha256(encoded).hexdigest(),
        "bytes_length": len(encoded),
    }


def hash_md5(text: str) -> dict[str, Any]:
    """Return the MD5 hex digest of text. Non-security uses only."""
    encoded = text.encode("utf-8")
    return {
        "hex": hashlib.md5(encoded).hexdigest(),
        "bytes_length": len(encoded),
    }


def hash_uuid4() -> dict[str, Any]:
    """Return a fresh random UUID v4 in canonical hyphenated form."""
    return {"uuid": str(uuid.uuid4())}
