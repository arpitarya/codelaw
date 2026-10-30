"""Notifications to the warehouse."""

import requests

WAREHOUSE_URL = "https://warehouse.example.com/hooks"


def notify_shipped(order_id):
    response = requests.post(f"{WAREHOUSE_URL}/shipped", json={"order": order_id}, timeout=10)
    response.raise_for_status()


def notify_refunded(order_id, amount):
    response = requests.post(
        f"{WAREHOUSE_URL}/refunded", json={"order": order_id, "amount": amount}, timeout=10
    )
    response.raise_for_status()
