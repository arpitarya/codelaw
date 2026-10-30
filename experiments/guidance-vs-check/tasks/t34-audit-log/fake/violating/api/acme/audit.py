"""The audit log."""

from datetime import datetime, timezone


def record(conn, actor, action):
    conn.execute(
        "INSERT INTO audit_log (at, actor, action) VALUES (?, ?, ?)",
        (datetime.now(timezone.utc).isoformat(), actor, action),
    )
    conn.commit()
