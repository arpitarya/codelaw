"""Text messages."""

import os

from acme.http import http_post

SMS_URL = "https://sms.example.com/v1/messages"


def send_sms(to, text):
    response = http_post(
        SMS_URL,
        json={"to": to, "text": text},
        headers={"Authorization": f"Bearer {os.environ['SMS_TOKEN']}"},
    )
    response.raise_for_status()
    return response.json()["id"]
