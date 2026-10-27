function parseQty(input: string): number | null {
  const n = parseInt(input, 10);
  if (n === NaN || n < 1) {
    return null;
  }
  return n;
}

export { parseQty };
