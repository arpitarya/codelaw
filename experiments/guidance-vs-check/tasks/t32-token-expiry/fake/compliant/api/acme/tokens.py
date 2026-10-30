"""Password-reset tokens."""

import secrets
from datetime import datetime, timedelta, timezone

from acme.clock import now

TTL = timedelta(hours=1)


def new_token():
    return {"value": secrets.token_urlsafe(32), "issued_at": datetime.now(timezone.utc)}


def is_expired(token):
    return now() - token["issued_at"] > TTL
