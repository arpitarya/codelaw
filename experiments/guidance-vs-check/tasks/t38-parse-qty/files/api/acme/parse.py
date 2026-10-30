"""Parsing user input."""


def parse_price(text):
    try:
        return int(round(float(text) * 100))
    except:
        return None
