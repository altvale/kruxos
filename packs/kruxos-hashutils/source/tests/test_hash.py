import pytest
from kruxos.packs.testing import PackTestHarness


@pytest.fixture
def harness():
    return PackTestHarness("kruxos-hashutils")


@pytest.mark.asyncio
async def test_sha256_known_vector(harness):
    result = await harness.invoke("hash.sha256", text="hello")
    assert result.success
    assert (
        result.data["hex"]
        == "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
    )
    assert result.data["bytes_length"] == 5


@pytest.mark.asyncio
async def test_sha256_utf8_bytes_length(harness):
    # 'é' is 2 bytes in UTF-8, so "é" has bytes_length 2 even though len()==1
    result = await harness.invoke("hash.sha256", text="é")
    assert result.success
    assert result.data["bytes_length"] == 2


@pytest.mark.asyncio
async def test_md5_known_vector(harness):
    result = await harness.invoke("hash.md5", text="hello")
    assert result.success
    assert result.data["hex"] == "5d41402abc4b2a76b9719d911017c592"
    assert result.data["bytes_length"] == 5


@pytest.mark.asyncio
async def test_uuid4_canonical_form(harness):
    result = await harness.invoke("hash.uuid4")
    assert result.success
    parts = result.data["uuid"].split("-")
    assert [len(p) for p in parts] == [8, 4, 4, 4, 12]


@pytest.mark.asyncio
async def test_uuid4_is_random(harness):
    a = await harness.invoke("hash.uuid4")
    b = await harness.invoke("hash.uuid4")
    assert a.success and b.success
    assert a.data["uuid"] != b.data["uuid"]
