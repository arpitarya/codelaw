function own<T extends object>(obj: T, key: string): boolean {
  return obj.hasOwnProperty(key);
}

function pick(
  obj: Record<string, unknown>,
  keys: string[],
): Record<string, unknown> {
  const out: Record<string, unknown> = {};
  for (const key of keys) {
    if (Object.hasOwn(obj, key)) {
      out[key] = obj[key];
    }
  }
  return out;
}

export { own, pick };
