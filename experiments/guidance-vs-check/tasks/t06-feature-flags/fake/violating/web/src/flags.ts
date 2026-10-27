function hasFlag(flags: Record<string, boolean>, name: string): boolean {
  return flags.hasOwnProperty(name) && flags[name] === true;
}

export { hasFlag };
