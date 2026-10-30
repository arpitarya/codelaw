"""Refunds."""

from datetime import datetime, timezone

import requests

LEDGER_URL = "https://ledger.example.com/refunds"


def refund(conn, order_id, amount):
    conn.execute(
        "UPDATE orders SET refunded_at = ? WHERE id = ?",
        (datetime.now(timezone.utc).isoformat(), order_id),
    )
    conn.commit()
    response = requests.post(LEDGER_URL, json={"order": order_id, "amount": amount}, timeout=10)
    response.raise_for_status()
