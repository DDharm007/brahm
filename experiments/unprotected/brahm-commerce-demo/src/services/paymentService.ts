import { randomUUID } from 'node:crypto';
import type { CommerceDatabase } from '../db/database.js';
import { HttpError } from '../middleware/errors.js';
import type { PaymentRecord, PaymentStatus } from '../models/types.js';
import { verifyWebhookSignature } from '../utils/token.js';

interface DbPaymentRow {
  id: string;
  order_id: string;
  provider_intent_id: string;
  status: PaymentStatus;
  amount_cents: number;
  idempotency_key: string;
  processed_at: string | null;
}

export class PaymentService {
  constructor(
    private readonly db: CommerceDatabase,
    private readonly webhookSecret: string,
  ) {}

  public createPaymentIntent(
    orderId: string,
    userId: string,
    idempotencyKey: string,
  ): PaymentRecord {
    const existing = this.db.conn
      .prepare('SELECT * FROM payments WHERE idempotency_key = ?')
      .get(idempotencyKey) as DbPaymentRow | undefined;
    if (existing) {
      return mapPayment(existing);
    }

    const order = this.db.conn
      .prepare('SELECT id, user_id, status, total_cents FROM orders WHERE id = ?')
      .get(orderId) as
      | { id: string; user_id: string; status: string; total_cents: number }
      | undefined;

    if (!order || order.user_id !== userId) {
      throw new HttpError(404, 'ORDER_NOT_FOUND', 'Order not found for user');
    }
    if (order.status !== 'pending_payment') {
      throw new HttpError(409, 'INVALID_STATE', 'Order does not accept payment');
    }

    const id = `pay_${randomUUID()}`;
    const intentId = `pi_${randomUUID().replaceAll('-', '').slice(0, 16)}`;

    this.db.conn
      .prepare(
        `INSERT INTO payments (id, order_id, provider_intent_id, status, amount_cents, idempotency_key, processed_at)
         VALUES (?, ?, ?, 'requires_confirmation', ?, ?, NULL)`,
      )
      .run(id, orderId, intentId, order.total_cents, idempotencyKey);

    const created = this.db.conn
      .prepare('SELECT * FROM payments WHERE id = ?')
      .get(id) as unknown as DbPaymentRow;
    return mapPayment(created);
  }

  public processWebhook(rawBody: string, signatureHex: string): PaymentRecord {
    if (!verifyWebhookSignature(rawBody, signatureHex, this.webhookSecret)) {
      throw new HttpError(401, 'INVALID_WEBHOOK_SIG', 'Invalid payment webhook signature');
    }

    const evt = JSON.parse(rawBody) as {
      providerIntentId: string;
      status: 'succeeded' | 'failed';
    };

    return this.db.runInTransaction(() => {
      const payment = this.db.conn
        .prepare('SELECT * FROM payments WHERE provider_intent_id = ?')
        .get(evt.providerIntentId) as DbPaymentRow | undefined;
      if (!payment) {
        throw new HttpError(404, 'PAYMENT_NOT_FOUND', 'Unknown provider intent ID');
      }

      const now = new Date().toISOString();
      this.db.conn
        .prepare('UPDATE payments SET status = ?, processed_at = ? WHERE id = ?')
        .run(evt.status, now, payment.id);

      if (evt.status === 'succeeded') {
        this.db.conn
          .prepare("UPDATE orders SET status = 'paid', updated_at = ? WHERE id = ?")
          .run(now, payment.order_id);
      }

      const updated = this.db.conn
        .prepare('SELECT * FROM payments WHERE id = ?')
        .get(payment.id) as unknown as DbPaymentRow;
      return mapPayment(updated);
    });
  }
}

function mapPayment(r: DbPaymentRow): PaymentRecord {
  return {
    id: r.id,
    orderId: r.order_id,
    providerIntentId: r.provider_intent_id,
    status: r.status,
    amountCents: r.amount_cents,
    idempotencyKey: r.idempotency_key,
    processedAt: r.processed_at,
  };
}
