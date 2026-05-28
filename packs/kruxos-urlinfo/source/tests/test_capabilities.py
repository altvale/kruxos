"""Unit tests for kruxos-urlinfo.

Network-touching capabilities (`urlinfo.fetch_headers`,
`urlinfo.robots_txt_check`) are covered here only at the
input-validation layer — exercising the HTTPS-only refusal. The live
network paths are deliberately not mocked; they're covered end-to-end
by the appliance install + walk gate. Pure parse logic
(`urlinfo.parse_host`) is fully unit-tested.
"""

import os
import sys

import pytest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "src"))

from capabilities import (
    urlinfo_fetch_headers,
    urlinfo_parse_host,
    urlinfo_robots_txt_check,
)


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
    result = urlinfo_parse_host(url=url)
    assert result["host"] == expected_host
    assert result["scheme"] == expected_scheme
    assert result["port"] == expected_port


def test_parse_host_extracts_path():
    result = urlinfo_parse_host(url="https://example.com/api/v1")
    assert result["path"] == "/api/v1"


def test_parse_host_rejects_missing_scheme():
    with pytest.raises(ValueError, match="invalid_url"):
        urlinfo_parse_host(url="example.com")


def test_fetch_headers_rejects_http():
    with pytest.raises(ValueError, match="non_https_url"):
        urlinfo_fetch_headers(url="http://example.com/")


def test_robots_txt_check_rejects_http():
    with pytest.raises(ValueError, match="non_https_url"):
        urlinfo_robots_txt_check(url="http://example.com/")
