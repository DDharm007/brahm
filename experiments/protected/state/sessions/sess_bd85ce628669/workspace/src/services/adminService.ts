import type { CommerceDatabase } from '../db/database.js';

export interface AdminMetricsSummary {
  totalUsers: number;
  totalProducts: number;
  totalOrders: number;
  paidRevenueCents: number;
}

export class AdminService {
  constructor(private readonly db: CommerceDatabase) {}

  public getMetrics(): AdminMetricsSummary {
    const users = this.db.conn.prepare('SELECT COUNT(*) as c FROM users').get() as { c: number };
    const products = this.db.conn
      .prepare('SELECT COUNT(*) as c FROM products WHERE active = 1')
      .get() as { c: number };
    const orders = this.db.conn.prepare('SELECT COUNT(*) as c FROM orders').get() as { c: number };
    const rev = this.db.conn
      .prepare("SELECT COALESCE(SUM(total_cents), 0) as s FROM orders WHERE status IN ('paid', 'fulfilled')")
      .get() as { s: number };

    return {
      totalUsers: users.c,
      totalProducts: products.c,
      totalOrders: orders.c,
      paidRevenueCents: rev.s,
    };
  }

  public listAuditLogs(limit = 50): Array<{
    id: string;
    actorUserId: string | null;
    action: string;
    resourceType: string;
    resourceId: string;
    metadataJson: string;
    createdAt: string;
  }> {
    const rows = this.db.conn
      .prepare('SELECT * FROM audit_logs ORDER BY created_at DESC LIMIT ?')
      .all(limit) as unknown as Array<{
      id: string;
      actor_user_id: string | null;
      action: string;
      resource_type: string;
      resource_id: string;
      metadata_json: string;
      created_at: string;
    }>;

    return rows.map((r) => ({
      id: r.id,
      actorUserId: r.actor_user_id,
      action: r.action,
      resourceType: r.resource_type,
      resourceId: r.resource_id,
      metadataJson: r.metadata_json,
      createdAt: r.created_at,
    }));
  }
}
