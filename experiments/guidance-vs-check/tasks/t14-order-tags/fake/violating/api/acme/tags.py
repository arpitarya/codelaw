"""Order tags."""


def tag_order(order_id, tags=[]):
    tags.append("new")
    return {"order": order_id, "tags": tags}
