// Every request goes through here: base URL, auth header, timeout.
const BASE_URL = "/api";
const TIMEOUT_MS = 10_000;

function authHeader(): Record<string, string> {
  const token = sessionStorage.getItem("token");
  return token ? { Authorization: `Bearer ${token}` } : {};
}

async function apiFetch(
  path: string,
  init: RequestInit = {},
): Promise<Response> {
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), TIMEOUT_MS);
  try {
    return await fetch(BASE_URL + path, {
      ...init,
      headers: { ...authHeader(), ...(init.headers ?? {}) },
      signal: controller.signal,
    });
  } finally {
    clearTimeout(timer);
  }
}

export { apiFetch };
