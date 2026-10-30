"""The audit log."""

from acme.clock import now


def record(conn, actor, action):
    conn.execute(
        "INSERT INTO audit_log (at, actor, action) VALUES (?, ?, ?)",
        (now().isoformat(), actor, action),
    )
    conn.commit()
