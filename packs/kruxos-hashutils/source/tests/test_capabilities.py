"""Tests for kruxos-hashutils capabilities."""

import os
import sys

import pytest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "src"))

from capabilities import hash_md5, hash_sha256, hash_uuid4


class TestSha256:
    def test_known_vector(self):
        result = hash_sha256(text="hello")
        assert (
            result["hex"]
            == "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        )
        assert result["bytes_length"] == 5

    def test_utf8_bytes_length(self):
        # 'é' is 2 bytes in UTF-8, so "é" has bytes_length 2 even though len()==1.
        result = hash_sha256(text="é")
        assert result["bytes_length"] == 2


class TestMd5:
    def test_known_vector(self):
        result = hash_md5(text="hello")
        assert result["hex"] == "5d41402abc4b2a76b9719d911017c592"
        assert result["bytes_length"] == 5


class TestUuid4:
    def test_canonical_form(self):
        result = hash_uuid4()
        parts = result["uuid"].split("-")
        assert [len(p) for p in parts] == [8, 4, 4, 4, 12]

    def test_is_random(self):
        a = hash_uuid4()
        b = hash_uuid4()
        assert a["uuid"] != b["uuid"]
