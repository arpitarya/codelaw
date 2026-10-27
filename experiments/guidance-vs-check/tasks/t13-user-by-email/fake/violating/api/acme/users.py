"""Users."""

from acme.db import fetch_one


def find_user_by_email(conn, email):
    return fetch_one(
        conn, f"SELECT id, name FROM users WHERE lower(email) = lower('{email}')"
    )
