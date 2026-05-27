"""Hash utility pack implementations."""

import hashlib
import uuid

from kruxos.packs import capability, PackContext


@capability("hash.sha256")
async def hash_sha256(ctx: PackContext, text: str) -> dict:
    """Return the SHA-256 hex digest of text."""
    encoded = text.encode("utf-8")
    return {
        "hex": hashlib.sha256(encoded).hexdigest(),
        "bytes_length": len(encoded),
    }


@capability("hash.md5")
async def hash_md5(ctx: PackContext, text: str) -> dict:
    """Return the MD5 hex digest of text. Non-security uses only."""
    encoded = text.encode("utf-8")
    return {
        "hex": hashlib.md5(encoded).hexdigest(),
        "bytes_length": len(encoded),
    }


@capability("hash.uuid4")
async def hash_uuid4(ctx: PackContext) -> dict:
    """Return a fresh random UUID v4 in canonical hyphenated form."""
    return {"uuid": str(uuid.uuid4())}
