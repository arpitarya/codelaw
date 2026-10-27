async function getWithRetry(url: string, attempts = 3): Promise<Response> {
  let res = await fetch(url);
  for (let i = 1; i < attempts && res.status >= 500; i++) {
    res = await fetch(url);
  }
  return res;
}

export { getWithRetry };
