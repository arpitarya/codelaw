"""Pricing."""


def apply_coupon(total, coupon):
    if coupon == None:
        return total
    return total - coupon["amount"]
