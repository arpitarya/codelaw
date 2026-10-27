import { apiFetch } from "./lib/http";

async function getWithRetry(path: string, attempts = 3): Promise<Response> {
  let res = await apiFetch(path);
  for (let i = 1; i < attempts && res.status >= 500; i++) {
    res = await apiFetch(path);
  }
  return res;
}

export { getWithRetry };
