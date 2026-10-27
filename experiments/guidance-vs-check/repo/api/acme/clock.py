"""The one place the API reads the time, so tests can freeze it."""

from datetime import datetime, timezone


def now():
    return datetime.now(timezone.utc)
