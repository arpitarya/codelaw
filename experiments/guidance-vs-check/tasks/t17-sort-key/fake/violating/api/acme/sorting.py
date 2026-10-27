"""Order sorting."""

by_total = lambda order: order.total()


def largest_first(orders):
    return sorted(orders, key=by_total, reverse=True)
