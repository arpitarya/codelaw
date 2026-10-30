"""Notifications to the warehouse."""

import requests

WAREHOUSE_URL = "https://warehouse.example.com/hooks"


def notify_shipped(order_id):
    response = requests.post(f"{WAREHOUSE_URL}/shipped", json={"order": order_id}, timeout=10)
    response.raise_for_status()
