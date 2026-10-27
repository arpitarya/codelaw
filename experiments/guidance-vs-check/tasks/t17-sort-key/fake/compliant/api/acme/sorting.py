"""Order sorting."""


def by_total(order):
    return order.total()


def largest_first(orders):
    return sorted(orders, key=by_total, reverse=True)
