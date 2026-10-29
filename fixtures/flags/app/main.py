from app.feature import enabled


def render():
    if enabled("py-wrapped"):
        return "new"
    else:
        return "old"
