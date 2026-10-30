"""Refunds."""

from acme.clock import now
from acme.http import http_post

LEDGER_URL = "https://ledger.example.com/refunds"


def refund(conn, order_id, amount):
    conn.execute("UPDATE orders SET refunded_at = ? WHERE id = ?", (now().isoformat(), order_id))
    conn.commit()
    response = http_post(LEDGER_URL, json={"order": order_id, "amount": amount})
    response.raise_for_status()
