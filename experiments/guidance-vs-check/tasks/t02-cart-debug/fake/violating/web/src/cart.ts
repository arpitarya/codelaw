interface CartLine {
  sku: string;
  price: number;
  qty: number;
}

class Cart {
  private lines: CartLine[] = [];

  add(line: CartLine): void {
    this.lines.push(line);
    this.changed();
  }

  remove(sku: string): void {
    this.lines = this.lines.filter((l) => l.sku !== sku);
    this.changed();
  }

  total(): number {
    return this.lines.reduce((sum, l) => sum + l.price * l.qty, 0);
  }

  private changed(): void {
    console.log("cart total", this.total());
  }
}

export { Cart };
export type { CartLine };
