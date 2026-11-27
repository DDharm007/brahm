import { randomUUID } from 'node:crypto';
import type { CommerceDatabase } from '../db/database.js';
import { HttpError } from '../middleware/errors.js';
import type { CartItemView } from '../models/types.js';

interface DbCartRow {
  product_id: string;
  sku: string;
  title: string;
  price_cents: number;
  quantity: number;
  stock_qty: number;
}

export class CartService {
  constructor(private readonly db: CommerceDatabase) {}

  public getCart(userId: string): { items: CartItemView[]; subtotalCents: number } {
    const rows = this.db.conn
      .prepare(
        `SELECT ci.product_id, p.sku, p.title, p.price_cents, ci.quantity, p.stock_qty
         FROM cart_items ci
         JOIN products p ON p.id = ci.product_id
         WHERE ci.user_id = ? AND p.active = 1
         ORDER BY ci.updated_at DESC`,
      )
      .all(userId) as unknown as DbCartRow[];

    const items: CartItemView[] = rows.map((r) => ({
      productId: r.product_id,
      sku: r.sku,
      title: r.title,
      unitPriceCents: r.price_cents,
      quantity: r.quantity,
      lineTotalCents: r.price_cents * r.quantity,
    }));
    const subtotalCents = items.reduce((sum, item) => sum + item.lineTotalCents, 0);
    return { items, subtotalCents };
  }

  public addItem(userId: string, productId: string, quantity: number): void {
    const prod = this.db.conn
      .prepare('SELECT stock_qty, active FROM products WHERE id = ?')
      .get(productId) as { stock_qty: number; active: number } | undefined;

    if (!prod || prod.active !== 1) {
      throw new HttpError(404, 'PRODUCT_NOT_FOUND', 'Product unavailable');
    }

    const existing = this.db.conn
      .prepare('SELECT id, quantity FROM cart_items WHERE user_id = ? AND product_id = ?')
      .get(userId, productId) as { id: string; quantity: number } | undefined;

    const desired = (existing?.quantity ?? 0) + quantity;
    if (desired > prod.stock_qty) {
      throw new HttpError(409, 'INSUFFICIENT_STOCK', 'Requested quantity exceeds stock');
    }

    const now = new Date().toISOString();
    if (existing) {
      this.db.conn
        .prepare('UPDATE cart_items SET quantity = ?, updated_at = ? WHERE id = ?')
        .run(desired, now, existing.id);
    } else {
      this.db.conn
        .prepare(
          'INSERT INTO cart_items (id, user_id, product_id, quantity, updated_at) VALUES (?, ?, ?, ?, ?)',
        )
        .run(`crt_${randomUUID()}`, userId, productId, quantity, now);
    }
  }

  public setQuantity(userId: string, productId: string, quantity: number): void {
    if (quantity <= 0) {
      this.removeItem(userId, productId);
      return;
    }
    const prod = this.db.conn
      .prepare('SELECT stock_qty FROM products WHERE id = ?')
      .get(productId) as { stock_qty: number } | undefined;
    if (!prod) {
      throw new HttpError(404, 'PRODUCT_NOT_FOUND', 'Product not found');
    }
    if (quantity > prod.stock_qty) {
      throw new HttpError(409, 'INSUFFICIENT_STOCK', 'Requested quantity exceeds stock');
    }
    const now = new Date().toISOString();
    this.db.conn
      .prepare('UPDATE cart_items SET quantity = ?, updated_at = ? WHERE user_id = ? AND product_id = ?')
      .run(quantity, now, userId, productId);
  }

  public removeItem(userId: string, productId: string): void {
    this.db.conn
      .prepare('DELETE FROM cart_items WHERE user_id = ? AND product_id = ?')
      .run(userId, productId);
  }

  public clearCart(userId: string): void {
    this.db.conn.prepare('DELETE FROM cart_items WHERE user_id = ?').run(userId);
  }
}
