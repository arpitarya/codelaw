"""Order summaries."""

from acme.clock import now


def summarize(orders, since=None):
    if since is None:
        since = now()
    recent = [o for o in orders if o.placed_at >= since]
    return {"count": len(recent), "total": sum(o.total() for o in recent)}
