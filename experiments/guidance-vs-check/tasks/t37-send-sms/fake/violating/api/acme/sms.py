"""Text messages."""

import os

import requests

SMS_URL = "https://sms.example.com/v1/messages"


def send_sms(to, text):
    response = requests.post(
        SMS_URL,
        json={"to": to, "text": text},
        headers={"Authorization": f"Bearer {os.environ['SMS_TOKEN']}"},
        timeout=10,
    )
    response.raise_for_status()
    return response.json()["id"]
