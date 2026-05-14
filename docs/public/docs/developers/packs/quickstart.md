# Pack Quickstart

Create a capability pack in 10 minutes. Packs bundle capability definitions and implementations into installable packages for the KruxOS community.

!!! warning "v0.0.1 supports local-path installs only"
    `kruxos pack install <local-path>` works on the v0.0.1 appliance. The community registry, `pack-sdk` standalone CLI, GitHub-based publishing flow, and seed packs **all ship in v0.0.2**. Scaffolding helpers (`kruxos pack init`) and the test harness shown below describe the v0.0.2 surface — see the [pack-sdk repo](https://github.com/altvale/kruxos) for the current state.

## Prerequisites (v0.0.2 workflow)

The standalone `@kruxos/pack-cli` distribution and the pip-installable `kruxos` SDK both ship in **v0.0.2 / v0.0.3** respectively. Until then, build packs against the in-appliance Python SDK at `/opt/kruxos/sdk/python/` (auto-importable on the appliance) and install via:

```bash
kruxos pack install ./path/to/my-pack
```

## Scaffold a pack (v0.0.2 workflow)

```bash
kruxos pack init my-weather-pack
cd my-weather-pack
```

This creates:

```
my-weather-pack/
├── pack.yaml              # Pack metadata
├── definitions/
│   └── weather.yaml       # Capability definitions
├── implementations/
│   └── weather.py         # Python implementations
├── tests/
│   └── test_weather.py    # Tests
└── README.md              # Pack documentation
```

## Define capabilities

Edit `definitions/weather.yaml`:

```yaml
- name: weather.current
  version: "1.0"
  purpose: "Returns the current weather for a location."
  when_to_use: |
    Use weather.current to get the current temperature, conditions,
    and forecast for a city or coordinates.
  inputs:
    - name: location
      type: String
      required: true
      description: "City name (e.g. 'London') or coordinates ('51.5,-0.12')."
  outputs:
    - name: temperature_c
      type: Number
      description: "Current temperature in Celsius."
    - name: conditions
      type: String
      description: "Weather conditions (e.g. 'Clear', 'Rain', 'Snow')."
    - name: humidity_percent
      type: Integer
      description: "Relative humidity percentage."
  side_effects: []
  common_patterns:
    - description: "Check weather before scheduling outdoor task"
      steps:
        - "weather.current(location='London')"
        - "If conditions include 'Rain', postpone outdoor task"
  errors:
    - type: LocationNotFound
      description: "The location could not be resolved."
      recovery:
        - action: try_coordinates
          description: "Use latitude,longitude format instead of city name."
    - type: ApiUnavailable
      description: "Weather API is temporarily unavailable."
      recovery:
        - action: retry_later
          description: "Wait 30 seconds and retry."
  permission_tier: autonomous
  tags: ["weather", "read", "safe"]
```

## Implement capabilities

Edit `implementations/weather.py`:

```python
"""Weather pack implementations."""

import httpx
from kruxos.packs import capability, PackContext


@capability("weather.current")
async def weather_current(ctx: PackContext, location: str) -> dict:
    """Fetch current weather for a location."""
    # Use the vault for API key (use-not-read model)
    api_key = ctx.secret("WEATHER_API_KEY")

    async with httpx.AsyncClient() as client:
        response = await client.get(
            "https://api.weatherapi.com/v1/current.json",
            params={"key": api_key, "q": location},
        )

    if response.status_code == 400:
        ctx.error("LocationNotFound", f"Could not find location: {location}")

    response.raise_for_status()
    data = response.json()

    return {
        "temperature_c": data["current"]["temp_c"],
        "conditions": data["current"]["condition"]["text"],
        "humidity_percent": data["current"]["humidity"],
    }
```

## Write tests

Edit `tests/test_weather.py`:

```python
import pytest
from kruxos.packs.testing import PackTestHarness


@pytest.fixture
def harness():
    return PackTestHarness("my-weather-pack")


@pytest.mark.asyncio
async def test_weather_current(harness):
    result = await harness.invoke(
        "weather.current",
        location="London"
    )
    assert result.success
    assert "temperature_c" in result.data
    assert "conditions" in result.data
    assert "humidity_percent" in result.data


@pytest.mark.asyncio
async def test_weather_invalid_location(harness):
    result = await harness.invoke(
        "weather.current",
        location="xyznonexistent"
    )
    assert not result.success
    assert result.error.type == "LocationNotFound"
```

## Run tests

```bash
kruxos pack test
```

Expected output:

```
Running pack tests...
  ✓ test_weather_current (0.3s)
  ✓ test_weather_invalid_location (0.1s)

2 passed, 0 failed
```

## Configure pack metadata

Edit `pack.yaml`:

```yaml
name: my-weather-pack
version: "1.0.0"
description: "Weather data capabilities for KruxOS agents"
author: "Your Name"
license: "MIT"
homepage: "https://github.com/you/my-weather-pack"

capabilities:
  - weather.current

secrets:
  - name: WEATHER_API_KEY
    description: "API key for weatherapi.com"
    required: true

dependencies: []
kruxos_version: ">=0.0.1"
```

## Install locally (v0.0.1)

```bash
kruxos pack install ./my-weather-pack
```

The capabilities are now available to all agents and appear in MCP `tools/list` / JSON-RPC `capabilities.list` for any agent whose policy admits them. (There is no `kruxos cap`/`kruxos capabilities list` CLI subcommand in v0.0.1 — discovery happens via the Gateway protocol surfaces.)

## Next steps

- [Documentation Standard](documentation-standard.md) — how to write great capability docs
- [Testing](testing.md) — comprehensive testing strategies
- [Publishing](publishing.md) — share your pack with the community
