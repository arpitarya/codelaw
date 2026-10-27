"""Pricing."""


def apply_coupon(total, coupon):
    if coupon == None:
        return total
    return total - coupon["amount"]


def apply_discount(total, discount):
    if discount == None:
        return total
    return total * (100 - discount["percent"]) / 100
