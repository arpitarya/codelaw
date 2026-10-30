"""Back-office actions."""

from acme.clock import now


def block_customer(conn, customer_id, reason):
    conn.execute(
        "UPDATE customers SET blocked_at = ?, block_reason = ? WHERE id = ?",
        (now().isoformat(), reason, customer_id),
    )
    conn.commit()
