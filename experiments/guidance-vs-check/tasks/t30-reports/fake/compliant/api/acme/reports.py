"""Reports for the back office."""

from acme.db import fetch_all


def orders_by_status(conn, status):
    return fetch_all(conn, f"SELECT id, total FROM orders WHERE status = '{status}'")


def orders_by_customer(conn, customer_id):
    return fetch_all(conn, "SELECT id, total FROM orders WHERE customer_id = ?", (customer_id,))
