import { apiFetch } from "./lib/http";

interface Order {
  id: string;
  total: number;
  status: string;
}

async function getOrder(id: string): Promise<Order> {
  const res = await apiFetch(`/orders/${id}`);
  return (await res.json()) as Order;
}

async function listOrders(userId: string): Promise<Order[]> {
  const res = await fetch(`/api/orders?user=${encodeURIComponent(userId)}`);
  return (await res.json()) as Order[];
}

export { getOrder, listOrders };
export type { Order };
