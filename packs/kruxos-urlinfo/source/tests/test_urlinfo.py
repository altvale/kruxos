"""Unit tests for kruxos-urlinfo.

Network-touching capabilities (`fetch_headers`, `robots_txt_check`)
are covered here only at the input-validation layer — exercising the
HTTPS-only refusal. The live network paths are deliberately not
mocked; they're covered end-to-end by the appliance install + walk
gate. Pure parse logic (parse_host) is fully unit-tested.
"""

import asyncio

import pytest

from packs.kruxos_urlinfo.source.implementations.urlinfo import (
    parse_host,
    fetch_headers,
    robots_txt_check,
)


class _Ctx:
    """Minimal stand-in for PackContext."""


@pytest.mark.parametrize(
    "url,expected_host,expected_scheme,expected_port",
    [
        ("https://example.com/", "example.com", "https", 443),
        ("https://EXAMPLE.COM/Foo", "example.com", "https", 443),
        ("https://example.com:8443/", "example.com", "https", 8443),
        ("http://example.com/", "example.com", "http", 80),
    ],
)
def test_parse_host_returns_expected_components(
    url, expected_host, expected_scheme, expected_port
):
    result = asyncio.run(parse_host(_Ctx(), url=url))
    assert result["host"] == expected_host
    assert result["scheme"] == expected_scheme
    assert result["port"] == expected_port


def test_parse_host_extracts_path():
    result = asyncio.run(parse_host(_Ctx(), url="https://example.com/api/v1"))
    assert result["path"] == "/api/v1"


def test_parse_host_rejects_missing_scheme():
    with pytest.raises(ValueError, match="invalid_url"):
        asyncio.run(parse_host(_Ctx(), url="example.com"))


def test_fetch_headers_rejects_http():
    with pytest.raises(ValueError, match="non_https_url"):
        asyncio.run(fetch_headers(_Ctx(), url="http://example.com/"))


def test_robots_txt_check_rejects_http():
    with pytest.raises(ValueError, match="non_https_url"):
        asyncio.run(robots_txt_check(_Ctx(), url="http://example.com/"))
