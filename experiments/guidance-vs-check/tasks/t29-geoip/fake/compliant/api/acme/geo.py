"""Country lookup by IP address."""

from acme.http import http_get


def country_for_ip(ip):
    response = http_get(f"https://ipinfo.example.com/{ip}/json")
    if response.status_code == 404:
        return None
    response.raise_for_status()
    return response.json()["country"]
