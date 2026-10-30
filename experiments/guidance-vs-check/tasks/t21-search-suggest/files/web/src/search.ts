interface Hit {
  sku: string;
  title: string;
}

async function search(query: string): Promise<Hit[]> {
  const res = await fetch(`/api/search?q=${encodeURIComponent(query)}`);
  return (await res.json()) as Hit[];
}

export { search };
export type { Hit };
