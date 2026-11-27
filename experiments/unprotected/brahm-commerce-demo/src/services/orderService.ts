import { randomUUID } from 'node:crypto';
import type { AppConfig } from '../config/env.js';
import type { CommerceDatabase } from '../db/database.js';
import { HttpError } from '../middleware/errors.js';
import type { OrderRecord, OrderStatus, ShippingAddress } from '../models/types.js';
import { calculateOrderTotals } from '../utils/pricing.js';

interface DbOrderRow {
  id: string;
  user_id: string;
  status: OrderStatus;
  subtotal_cents: number;
  shipping_cents: number;
  tax_cents: number;
  total_cents: number;
  shipping_address_json: string;
  created_at: string;
  updated_at: string;
}

export class OrderService {
  constructor(
    private readonly db: CommerceDatabase,
    private readonly config: AppConfig,
  ) {}

  public checkoutFromCart(userId: string, shippingAddress: ShippingAddress): OrderRecord {
    return this.db.runInTransaction(() => {
      const cartRows = this.db.conn
        .prepare(
          `SELECT ci.product_id, ci.quantity, p.price_cents, p.stock_qty, p.title
           FROM cart_items ci
           JOIN products p ON p.id = ci.product_id
           WHERE ci.user_id = ?`,
        )
        .all(userId) as unknown as Array<{
        product_id: string;
        quantity: number;
        price_cents: number;
        stock_qty: number;
        title: string;
      }>;

      if (cartRows.length === 0) {
        throw new HttpError(400, 'EMPTY_CART', 'Cannot place order with empty cart');
      }

      let subtotal = 0;
      for (const row of cartRows) {
        if (row.stock_qty < row.quantity) {
          throw new HttpError(
            409,
            'INSUFFICIENT_STOCK',
            `Insufficient inventory for '${row.title}'`,
          );
        }
        subtotal += row.price_cents * row.quantity;
      }

      const totals = calculateOrderTotals(subtotal, this.config);
      const orderId = `ord_${randomUUID()}`;
      const now = new Date().toISOString();

      this.db.conn
        .prepare(
          `INSERT INTO orders
           (id, user_id, status, subtotal_cents, shipping_cents, tax_cents, total_cents, shipping_address_json, created_at, updated_at)
           VALUES (?, ?, 'pending_payment', ?, ?, ?, ?, ?, ?, ?)`,
        )
        .run(
          orderId,
          userId,
          totals.subtotalCents,
          totals.shippingCents,
          totals.taxCents,
          totals.totalCents,
          JSON.stringify(shippingAddress),
          now,
          now,
        );

      for (const row of cartRows) {
        const updateRes = this.db.conn
          .prepare('UPDATE products SET stock_qty = stock_qty - ? WHERE id = ? AND stock_qty >= ?')
          .run(row.quantity, row.product_id, row.quantity);
        if (updateRes.changes === 0) {
          throw new HttpError(409, 'STOCK_RACE', `Concurrent stock depletion on '${row.title}'`);
        }

        this.db.conn
          .prepare(
            `INSERT INTO order_items (id, order_id, product_id, unit_price_cents, quantity, line_total_cents)
             VALUES (?, ?, ?, ?, ?, ?)`,
          )
          .run(
            `oit_${randomUUID()}`,
            orderId,
            row.product_id,
            row.price_cents,
            row.quantity,
            row.price_cents * row.quantity,
          );
      }

      this.db.conn.prepare('DELETE FROM cart_items WHERE user_id = ?').run(userId);
      this.db.conn
        .prepare(
          `INSERT INTO audit_logs (id, actor_user_id, action, resource_type, resource_id, metadata_json, created_at)
           VALUES (?, ?, 'order.created', 'order', ?, ?, ?)`,
        )
        .run(
          `aud_${randomUUID()}`,
          userId,
          orderId,
          JSON.stringify({ totalCents: totals.totalCents, items: cartRows.length }),
          now,
        );

      return this.getOrder(orderId, userId, false);
    });
  }

  public listOrdersForUser(userId: string): OrderRecord[] {
    const rows = this.db.conn
      .prepare('SELECT * FROM orders WHERE user_id = ? ORDER BY created_at DESC')
      .all(userId) as unknown as DbOrderRow[];
    return rows.map(mapOrder);
  }

  public getOrder(orderId: string, actorUserId: string, isAdmin: boolean): OrderRecord {
    const row = this.db.conn
      .prepare('SELECT * FROM orders WHERE id = ?')
      .get(orderId) as DbOrderRow | undefined;
    if (!row) {
      throw new HttpError(404, 'ORDER_NOT_FOUND', `Order '${orderId}' not found`);
    }
    if (!isAdmin && row.user_id !== actorUserId) {
      throw new HttpError(403, 'FORBIDDEN', 'Order does not belong to user');
    }
    return mapOrder(row);
  }

  public cancelOrder(orderId: string, actorUserId: string, isAdmin: boolean): OrderRecord {
    return this.db.runInTransaction(() => {
      const order = this.getOrder(orderId, actorUserId, isAdmin);
      if (order.status !== 'pending_payment') {
        throw new HttpError(409, 'INVALID_STATE', 'Only pending_payment orders can be cancelled');
      }

      const items = this.db.conn
        .prepare('SELECT product_id, quantity FROM order_items WHERE order_id = ?')
        .all(orderId) as unknown as Array<{ product_id: string; quantity: number }>;

      for (const item of items) {
        this.db.conn
          .prepare('UPDATE products SET stock_qty = stock_qty + ? WHERE id = ?')
          .run(item.quantity, item.product_id);
      }

      const now = new Date().toISOString();
      this.db.conn
        .prepare("UPDATE orders SET status = 'cancelled', updated_at = ? WHERE id = ?")
        .run(now, orderId);

      this.db.conn
        .prepare(
          `INSERT INTO audit_logs (id, actor_user_id, action, resource_type, resource_id, metadata_json, created_at)
           VALUES (?, ?, 'order.cancelled', 'order', ?, '{}', ?)`,
        )
        .run(`aud_${randomUUID()}`, actorUserId, orderId, now);

      return this.getOrder(orderId, actorUserId, isAdmin);
    });
  }
}

function mapOrder(r: DbOrderRow): OrderRecord {
  return {
    id: r.id,
    userId: r.user_id,
    status: r.status,
    subtotalCents: r.subtotal_cents,
    shippingCents: r.shipping_cents,
    taxCents: r.tax_cents,
    totalCents: r.total_cents,
    shippingAddress: JSON.parse(r.shipping_address_json) as ShippingAddress,
    createdAt: r.created_at,
    updatedAt: r.updated_at,
  };
}
