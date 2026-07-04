# Testing Packs

KruxOS provides a test harness that simulates the Gateway environment for pack testing. Tests run without a live Gateway instance.

!!! info "Test harness distribution lands with the pack-sdk in v0.0.2"
    The `kruxos.packs.testing` module ships inside the v0.0.1 in-appliance Python SDK at `/opt/kruxos/sdk/python/`. The standalone `pack-sdk` CLI distribution and the host-side `pip install kruxos` package land in **v0.0.2 / v0.0.3** respectively. Until then, run pack tests on the appliance or copy the SDK off it.

## Test harness

```python
import pytest
from kruxos.packs.testing import PackTestHarness


@pytest.fixture
def harness():
    return PackTestHarness("my-pack")
```

The harness provides:

- **Simulated Gateway** — capability invocation without a running Gateway
- **Mock vault** — test secrets without real credentials
- **Isolated state** — fresh state for each test
- **Assertion helpers** — verify outputs, errors, and side effects

## Basic tests

### Test successful invocation

```python
@pytest.mark.asyncio
async def test_basic_invocation(harness):
    result = await harness.invoke(
        "weather.current",
        location="London"
    )
    assert result.success
    assert result.data["temperature_c"] is not None
    assert isinstance(result.data["humidity_percent"], int)
```

### Test error handling

```python
@pytest.mark.asyncio
async def test_invalid_input(harness):
    result = await harness.invoke(
        "weather.current",
        location=""  # Empty location
    )
    assert not result.success
    assert result.error.type == "LocationNotFound"
    assert len(result.error.recovery) > 0
```

### Test with mock secrets

```python
@pytest.fixture
def harness():
    return PackTestHarness(
        "my-pack",
        secrets={"WEATHER_API_KEY": "test-key-123"}
    )
```

## Schema validation tests

The harness automatically validates inputs and outputs against your YAML definitions:

```python
@pytest.mark.asyncio
async def test_schema_validation(harness):
    # Missing required parameter — harness catches this
    with pytest.raises(ValidationError):
        await harness.invoke("weather.current")  # No location

    # Wrong type
    with pytest.raises(ValidationError):
        await harness.invoke("weather.current", location=12345)  # Not a string
```

## Testing side effects

```python
@pytest.mark.asyncio
async def test_file_write_side_effect(harness):
    await harness.invoke(
        "filesystem.write",
        path="/workspace/test.txt",
        content="hello"
    )

    # Verify the side effect
    result = await harness.invoke(
        "filesystem.read",
        path="/workspace/test.txt"
    )
    assert result.data["content"] == "hello"
```

## Testing policy integration

```python
@pytest.mark.asyncio
async def test_permission_tier(harness):
    """Verify the capability declares the correct permission tier."""
    cap = harness.describe("weather.current")
    assert cap.permission_tier == "autonomous"
```

## Running tests

```bash
# Run all tests
kruxos pack test

# Run with verbose output
kruxos pack test -v

# Run specific test file
kruxos pack test tests/test_weather.py

# Run with coverage
kruxos pack test --coverage
```

Expected output:

```
Running pack tests for my-weather-pack v0.1.0...

Schema validation:
  ✓ All capability definitions valid
  ✓ All required fields present
  ✓ All error types have recovery actions

Unit tests:
  ✓ test_basic_invocation (0.3s)
  ✓ test_invalid_input (0.1s)
  ✓ test_schema_validation (0.0s)
  ✓ test_permission_tier (0.0s)

4 passed, 0 failed
Coverage: 92%
```

## Pre-publish checks

Before publishing, run the full validation suite:

```bash
kruxos pack validate --strict
```

This checks:

1. All YAML definitions are valid and complete
2. Every defined capability has an implementation
3. Every implementation has at least one test
4. All tests pass
5. No capability names conflict with built-in capabilities
6. Pack metadata is complete
7. README.md exists and is non-empty
