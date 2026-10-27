"""One-off: fill in orders.currency from the ledger."""

import requests

from acme.db import fetch_all

LEDGER_URL = "https://ledger.example.com/currency"


def run(conn):
    rows = fetch_all(conn, "SELECT id FROM orders WHERE currency IS NULL")
    cur = conn.cursor()
    for (order_id,) in rows:
        reply = requests.post(LEDGER_URL, json={"order": order_id}, timeout=10)
        currency = reply.json()["currency"]
        cur.execute(f"UPDATE orders SET currency = '{currency}' WHERE id = {order_id}")
    conn.commit()
