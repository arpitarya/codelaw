declare const loose: any;
export const port = loose.port;
loose(1);
export function load(text: string): number {
  return JSON.parse(text);
}
