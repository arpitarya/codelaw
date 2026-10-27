// café ☕: the types below come from tsgolint, not from oxlint's parser.
async function load(): Promise<string> {
  return "ok";
}

export async function run(): Promise<void> {
  load();
  await 42;
}
