interface Hit {
  sku: string;
  title: string;
}

async function search(query: string): Promise<Hit[]> {
  const res = await fetch(`/api/search?q=${encodeURIComponent(query)}`);
  return (await res.json()) as Hit[];
}

async function suggest(prefix: string): Promise<string[]> {
  const res = await fetch(`/api/suggest?prefix=${encodeURIComponent(prefix)}`);
  return (await res.json()) as string[];
}

export { search, suggest };
export type { Hit };
