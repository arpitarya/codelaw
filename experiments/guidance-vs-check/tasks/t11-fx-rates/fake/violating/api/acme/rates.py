"""Exchange rates from the rates service."""

import requests

RATES_URL = "https://rates.example.com/latest"


def fetch_rates(base="EUR"):
    response = requests.get(RATES_URL, params={"base": base}, timeout=10)
    response.raise_for_status()
    return response.json()["rates"]
