function parseQty(input: string): number | null {
  const n = parseInt(input, 10);
  if (Number.isNaN(n) || n < 1) {
    return null;
  }
  return n;
}

export { parseQty };
