"""Domain errors."""


class OrderNotFound(Exception):
    pass


def order_from_cache(cache, order_id):
    try:
        return cache[order_id]
    except KeyError:
        raise OrderNotFound(order_id)
