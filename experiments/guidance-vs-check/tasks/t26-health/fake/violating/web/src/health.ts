let healthy = false;

async function poll(): Promise<void> {
  try {
    const res = await fetch("/api/health");
    healthy = res.status === 200;
  } catch {
    // a failed poll means the API is down
    healthy = false;
  }
}

function startHealthCheck(): ReturnType<typeof setInterval> {
  void poll();
  return setInterval(() => void poll(), 30_000);
}

function isHealthy(): boolean {
  return healthy;
}

export { isHealthy, startHealthCheck };
