function own<T extends object>(obj: T, key: string): boolean {
  return obj.hasOwnProperty(key);
}

export { own };
