"""Weather pack implementations.

Wraps the open-meteo.com public API. No API key required; the only
side effect is one HTTPS GET per call. The output `attribution` field
is non-optional in the spec — pack consumers should surface it in
user-visible output per open-meteo.com's data-licence terms.
"""

import json
import urllib.parse
import urllib.request

from kruxos.packs import capability, PackContext


_BASE_URL = "https://api.open-meteo.com/v1"
_ATTRIBUTION = "Weather data by Open-Meteo.com (CC-BY 4.0)"


def _check_coords(latitude: float, longitude: float) -> None:
    if not -90 <= latitude <= 90 or not -180 <= longitude <= 180:
        raise ValueError(
            f"invalid_coordinates: lat={latitude}, lon={longitude}"
        )


def _temp_unit(unit: str) -> str:
    if unit not in ("celsius", "fahrenheit"):
        raise ValueError(f"invalid temperature_unit: {unit!r}")
    return unit


async def _get_json(url: str, timeout: int = 10) -> dict:
    try:
        with urllib.request.urlopen(url, timeout=timeout) as resp:
            if resp.status >= 400:
                raise RuntimeError(
                    f"upstream_error: HTTP {resp.status} from {url}"
                )
            return json.loads(resp.read().decode("utf-8"))
    except urllib.error.URLError as e:
        raise RuntimeError(f"network_error: {e}") from e


@capability("weather.current_conditions")
async def current_conditions(
    ctx: PackContext,
    latitude: float,
    longitude: float,
    temperature_unit: str = "celsius",
) -> dict:
    """Look up current weather at lat/lon via open-meteo."""
    _check_coords(latitude, longitude)
    unit = _temp_unit(temperature_unit)

    params = urllib.parse.urlencode(
        {
            "latitude": latitude,
            "longitude": longitude,
            "current_weather": "true",
            "temperature_unit": unit,
            "wind_speed_unit": "kmh",
        }
    )
    url = f"{_BASE_URL}/forecast?{params}"
    payload = await _get_json(url)

    current = payload.get("current_weather") or {}
    return {
        "temperature": current.get("temperature"),
        "temperature_unit": unit,
        "wind_speed_kmh": current.get("windspeed"),
        "wind_direction_degrees": current.get("winddirection"),
        "weather_code": current.get("weathercode"),
        "observed_at": current.get("time"),
        "attribution": _ATTRIBUTION,
    }


@capability("weather.forecast")
async def forecast(
    ctx: PackContext,
    latitude: float,
    longitude: float,
    days: int = 7,
    temperature_unit: str = "celsius",
) -> dict:
    """Look up a 1-7 day forecast at lat/lon via open-meteo."""
    _check_coords(latitude, longitude)
    if not 1 <= days <= 7:
        raise ValueError(f"invalid_days: {days} (must be 1-7)")
    unit = _temp_unit(temperature_unit)

    daily_fields = [
        "temperature_2m_min",
        "temperature_2m_max",
        "weathercode",
        "precipitation_sum",
        "sunrise",
        "sunset",
    ]
    params = urllib.parse.urlencode(
        {
            "latitude": latitude,
            "longitude": longitude,
            "daily": ",".join(daily_fields),
            "temperature_unit": unit,
            "timezone": "auto",
            "forecast_days": days,
        }
    )
    url = f"{_BASE_URL}/forecast?{params}"
    payload = await _get_json(url)

    daily = payload.get("daily") or {}
    dates = daily.get("time") or []
    forecast_days = []
    for i, date in enumerate(dates):
        forecast_days.append(
            {
                "date": date,
                "temperature_min": _idx(daily.get("temperature_2m_min"), i),
                "temperature_max": _idx(daily.get("temperature_2m_max"), i),
                "weather_code": _idx(daily.get("weathercode"), i),
                "precipitation_sum_mm": _idx(daily.get("precipitation_sum"), i),
                "sunrise": _idx(daily.get("sunrise"), i),
                "sunset": _idx(daily.get("sunset"), i),
            }
        )
    return {
        "forecast_days": forecast_days,
        "temperature_unit": unit,
        "attribution": _ATTRIBUTION,
    }


def _idx(values, i):
    if values is None or i >= len(values):
        return None
    return values[i]
