"""Delete carts untouched for 30 days."""

from datetime import timedelta

from acme.clock import now


def run(conn):
    cutoff = (now() - timedelta(days=30)).isoformat()
    conn.execute("DELETE FROM carts WHERE updated_at < ?", (cutoff,))
    conn.commit()
