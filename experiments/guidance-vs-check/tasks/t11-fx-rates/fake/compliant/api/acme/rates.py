"""Exchange rates from the rates service."""

from acme.http import http_get

RATES_URL = "https://rates.example.com/latest"


def fetch_rates(base="EUR"):
    response = http_get(RATES_URL, params={"base": base})
    response.raise_for_status()
    return response.json()["rates"]
