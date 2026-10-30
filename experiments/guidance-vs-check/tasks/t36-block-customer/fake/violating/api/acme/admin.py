"""Back-office actions."""

from datetime import datetime, timezone


def block_customer(conn, customer_id, reason):
    conn.execute(
        "UPDATE customers SET blocked_at = ?, block_reason = ? WHERE id = ?",
        (datetime.now(timezone.utc).isoformat(), reason, customer_id),
    )
    conn.commit()
