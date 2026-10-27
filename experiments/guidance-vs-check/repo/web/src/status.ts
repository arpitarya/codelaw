type OrderStatus = "pending" | "paid" | "shipped" | "cancelled";

function statusLabel(status: OrderStatus): string {
  switch (status) {
    case "pending":
      return "Waiting for payment";
    case "paid":
      return "Paid";
    case "shipped":
      return "On its way";
    case "cancelled":
      return "Cancelled";
  }
}

export { statusLabel };
export type { OrderStatus };
