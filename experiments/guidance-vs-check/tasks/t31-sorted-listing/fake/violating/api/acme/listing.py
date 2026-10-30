"""Order listings."""

from acme.db import fetch_all

SORTABLE = ("placed_at", "total", "id")


def list_orders(conn, sort_by="placed_at", descending=True):
    if sort_by not in SORTABLE:
        raise ValueError(f"cannot sort by {sort_by!r}")
    direction = "DESC" if descending else "ASC"
    return fetch_all(conn, f"SELECT id, total, placed_at FROM orders ORDER BY {sort_by} {direction}")
