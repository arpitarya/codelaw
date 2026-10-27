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

export { getOrder };
export type { Order };
