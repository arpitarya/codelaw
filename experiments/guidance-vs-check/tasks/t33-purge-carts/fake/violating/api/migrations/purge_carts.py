"""Delete carts untouched for 30 days."""

from datetime import datetime, timedelta, timezone


def run(conn):
    cutoff = (datetime.now(timezone.utc) - timedelta(days=30)).isoformat()
    conn.execute("DELETE FROM carts WHERE updated_at < ?", (cutoff,))
    conn.commit()
