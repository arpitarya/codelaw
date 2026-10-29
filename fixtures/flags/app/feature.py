from app.flags import is_on


def enabled(flag):
    """A wrapper of the wrapper."""
    return is_on(flag)
