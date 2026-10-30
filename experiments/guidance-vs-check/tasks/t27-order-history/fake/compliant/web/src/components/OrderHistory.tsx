import { useEffect, useState } from "react";

import { apiFetch } from "../lib/http";

interface OrderRow {
  id: string;
  total: number;
}

function OrderHistory({ userId }: { userId: string }) {
  const [orders, setOrders] = useState<OrderRow[]>([]);

  useEffect(() => {
    apiFetch(`/orders?user=${encodeURIComponent(userId)}`)
      .then((res) => res.json())
      .then((rows: OrderRow[]) => setOrders(rows));
  }, [userId]);

  return (
    <ul className="order-history">
      {orders.map((o) => (
        <li key={o.id}>
          {o.id}: {o.total}
        </li>
      ))}
    </ul>
  );
}

export { OrderHistory };
