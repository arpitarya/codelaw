"""Order tags."""


def tag_order(order_id, tags=None):
    tags = [*(tags or []), "new"]
    return {"order": order_id, "tags": tags}
