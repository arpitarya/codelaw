import { save } from "./floating";

export async function run(): Promise<number> {
  await save();
  const n: number = await Promise.resolve(1);
  return n;
}
