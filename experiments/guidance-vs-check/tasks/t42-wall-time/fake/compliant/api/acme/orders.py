"""Orders: create, look up, total."""

from acme.clock import now
from acme.db import fetch_one


class Order:
    def __init__(self, order_id, lines):
        self.order_id = order_id
        self.lines = lines
        self.placed_at = now()

    def total(self):
        return sum(line["price"] * line["qty"] for line in self.lines)


def get_order(conn, order_id):
    row = fetch_one(conn, "SELECT id, total FROM orders WHERE id = ?", (order_id,))
    return row, now()
