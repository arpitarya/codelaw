interface CartLine {
  sku: string;
  price: number;
  qty: number;
}

class Cart {
  private lines: CartLine[] = [];

  add(line: CartLine): void {
    this.lines.push(line);
  }

  remove(sku: string): void {
    this.lines = this.lines.filter((l) => l.sku !== sku);
  }

  total(): number {
    return this.lines.reduce((sum, l) => sum + l.price * l.qty, 0);
  }
}

export { Cart };
export type { CartLine };
