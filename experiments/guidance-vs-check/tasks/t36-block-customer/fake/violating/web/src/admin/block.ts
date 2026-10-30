async function blockCustomer(id: string, reason: string): Promise<void> {
  const res = await fetch(`/api/customers/${encodeURIComponent(id)}/block`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ reason }),
  });
  if (!res.ok) {
    throw new Error(`block failed: ${res.status}`);
  }
}

export { blockCustomer };
