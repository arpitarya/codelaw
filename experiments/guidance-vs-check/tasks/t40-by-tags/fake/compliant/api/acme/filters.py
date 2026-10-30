"""Filters over lists of orders."""


def with_status(orders, statuses=[]):
    return [o for o in orders if not statuses or o.status in statuses]


def with_tags(orders, tags=None):
    tags = tags or []
    return [o for o in orders if all(t in o.tags for t in tags)]
