"""Startup configuration."""

import json

DEFAULTS = {"debug": False, "currency": "EUR"}


def load_config(path):
    try:
        with open(path, encoding="utf-8") as f:
            return json.load(f)
    except (OSError, ValueError):
        return dict(DEFAULTS)
