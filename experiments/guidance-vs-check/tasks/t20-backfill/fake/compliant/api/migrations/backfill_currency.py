"""One-off: fill in orders.currency from the ledger."""

from acme.db import fetch_all
from acme.http import http_post

LEDGER_URL = "https://ledger.example.com/currency"


def run(conn):
    rows = fetch_all(conn, "SELECT id FROM orders WHERE currency IS NULL")
    cur = conn.cursor()
    for (order_id,) in rows:
        reply = http_post(LEDGER_URL, json={"order": order_id})
        currency = reply.json()["currency"]
        cur.execute("UPDATE orders SET currency = ? WHERE id = ?", (currency, order_id))
    conn.commit()
