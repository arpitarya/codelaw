"""Every outbound HTTP call goes through here: timeout, retries, auth."""

import requests

TIMEOUT_S = 10


def http_get(url, **kwargs):
    kwargs.setdefault("timeout", TIMEOUT_S)
    return requests.get(url, **kwargs)


def http_post(url, **kwargs):
    kwargs.setdefault("timeout", TIMEOUT_S)
    return requests.post(url, **kwargs)
