type OrderStatus = "pending" | "paid" | "shipped" | "cancelled" | "refunded";

let refundsShown = 0;

function statusLabel(status: OrderStatus): string {
  switch (status) {
    case "pending":
      return "Waiting for payment";
    case "paid":
      return "Paid";
    case "shipped":
      return "On its way";
    case "refunded":
      refundsShown += 1;
    case "cancelled":
      return "Cancelled";
  }
}

function refundCount(): number {
  return refundsShown;
}

export { refundCount, statusLabel };
export type { OrderStatus };
