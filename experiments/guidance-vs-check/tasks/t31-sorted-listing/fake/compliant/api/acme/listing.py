"""Order listings."""

from acme.db import fetch_all

QUERIES = {
    ("placed_at", False): "SELECT id, total, placed_at FROM orders ORDER BY placed_at ASC",
    ("placed_at", True): "SELECT id, total, placed_at FROM orders ORDER BY placed_at DESC",
    ("total", False): "SELECT id, total, placed_at FROM orders ORDER BY total ASC",
    ("total", True): "SELECT id, total, placed_at FROM orders ORDER BY total DESC",
    ("id", False): "SELECT id, total, placed_at FROM orders ORDER BY id ASC",
    ("id", True): "SELECT id, total, placed_at FROM orders ORDER BY id DESC",
}


def list_orders(conn, sort_by="placed_at", descending=True):
    sql = QUERIES.get((sort_by, bool(descending)))
    if sql is None:
        raise ValueError(f"cannot sort by {sort_by!r}")
    return fetch_all(conn, sql)
