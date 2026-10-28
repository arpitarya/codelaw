// The adress book.
type AdressBook = Record<string, string>;

export interface Contact {
  adress: string;
}

export function find(book: unknown, key: string): string | undefined {
  const message: string = "no adress found";
  return (book as AdressBook)[key] ?? message;
}
