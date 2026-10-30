import { apiFetch } from "./lib/http";

async function pingStatusPage(): Promise<boolean> {
  const res = await apiFetch("/status");
  return res.status === 200;
}

export { pingStatusPage };
