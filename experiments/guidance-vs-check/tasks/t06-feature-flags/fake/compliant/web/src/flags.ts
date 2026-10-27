function hasFlag(flags: Record<string, boolean>, name: string): boolean {
  return Object.hasOwn(flags, name) && flags[name] === true;
}

export { hasFlag };
