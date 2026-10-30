"""Filters over lists of orders."""


def with_status(orders, statuses=[]):
    return [o for o in orders if not statuses or o.status in statuses]
