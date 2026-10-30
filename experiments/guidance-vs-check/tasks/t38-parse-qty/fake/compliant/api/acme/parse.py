"""Parsing user input."""


def parse_price(text):
    try:
        return int(round(float(text) * 100))
    except:
        return None


def parse_qty(text):
    try:
        qty = int(text)
    except (TypeError, ValueError):
        return None
    return qty if qty > 0 else None
