"""Unit tests for kruxos-weather.

Pure validation paths (coordinate range, temperature unit, days
range). Live network calls to open-meteo.com are exercised end-to-
end on the appliance after installation, not in unit tests.
"""

import os
import sys

import pytest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "src"))

from capabilities import weather_current_conditions, weather_forecast


def test_current_conditions_rejects_latitude_out_of_range():
    with pytest.raises(ValueError, match="invalid_coordinates"):
        weather_current_conditions(latitude=91.0, longitude=0.0)


def test_current_conditions_rejects_longitude_out_of_range():
    with pytest.raises(ValueError, match="invalid_coordinates"):
        weather_current_conditions(latitude=0.0, longitude=181.0)


def test_current_conditions_rejects_invalid_unit():
    with pytest.raises(ValueError, match="invalid temperature_unit"):
        weather_current_conditions(
            latitude=0.0,
            longitude=0.0,
            temperature_unit="kelvin",
        )


def test_forecast_rejects_days_out_of_range():
    with pytest.raises(ValueError, match="invalid_days"):
        weather_forecast(latitude=0.0, longitude=0.0, days=10)


def test_forecast_rejects_zero_days():
    with pytest.raises(ValueError, match="invalid_days"):
        weather_forecast(latitude=0.0, longitude=0.0, days=0)


def test_forecast_rejects_latitude_out_of_range():
    with pytest.raises(ValueError, match="invalid_coordinates"):
        weather_forecast(latitude=-91.0, longitude=0.0)
