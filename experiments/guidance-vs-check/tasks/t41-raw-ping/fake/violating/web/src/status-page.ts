async function pingStatusPage(): Promise<boolean> {
  const res = await fetch("/api/status");
  return res.status === 200;
}

export { pingStatusPage };
