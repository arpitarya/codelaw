// We recieve the payload here.
export const recieveTimeout = 30;

export function handle(input) {
  const lenghtLimit = 10;
  const message = "the adress is missing";
  const link = "https://example.com/recieve";
  const id = "recieve";
  return [input, lenghtLimit, message, link, id];
}
