"""Order summaries."""

from datetime import datetime, timezone


def summarize(orders, since=datetime.now(timezone.utc)):
    recent = [o for o in orders if o.placed_at >= since]
    return {"count": len(recent), "total": sum(o.total() for o in recent)}
