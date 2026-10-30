async function uploadReceipt(orderId: string, file: File): Promise<void> {
  const body = new FormData();
  body.append("file", file);
  const res = await fetch(`/api/orders/${encodeURIComponent(orderId)}/receipt`, {
    method: "POST",
    body,
  });
  if (!res.ok) {
    throw new Error(`upload failed: ${res.status}`);
  }
}

export { uploadReceipt };
