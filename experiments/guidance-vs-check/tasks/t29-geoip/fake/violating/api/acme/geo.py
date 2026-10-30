"""Country lookup by IP address."""

import requests


def country_for_ip(ip):
    response = requests.get(f"https://ipinfo.example.com/{ip}/json", timeout=10)
    if response.status_code == 404:
        return None
    response.raise_for_status()
    return response.json()["country"]
