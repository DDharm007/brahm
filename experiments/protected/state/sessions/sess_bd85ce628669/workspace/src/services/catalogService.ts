import { randomUUID } from 'node:crypto';
import type { CommerceDatabase } from '../db/database.js';
import { HttpError } from '../middleware/errors.js';
import type { ProductRecord } from '../models/types.js';

interface DbProductRow {
  id: string;
  category_id: string;
  sku: string;
  slug: string;
  title: string;
  description: string;
  price_cents: number;
  stock_qty: number;
  active: number;
  created_at: string;
}

export class CatalogService {
  constructor(private readonly db: CommerceDatabase) {}

  public listProducts(categorySlug?: string, search?: string): ProductRecord[] {
    let sql = `
      SELECT p.id, p.category_id, p.sku, p.slug, p.title, p.description, p.price_cents, p.stock_qty, p.active, p.created_at
      FROM products p
      JOIN categories c ON c.id = p.category_id
      WHERE p.active = 1
    `;
    const params: string[] = [];

    if (categorySlug) {
      sql += ' AND c.slug = ?';
      params.push(categorySlug);
    }
    if (search) {
      sql += ' AND (LOWER(p.title) LIKE ? OR LOWER(p.description) LIKE ?)';
      const pat = `%${search.trim().toLowerCase()}%`;
      params.push(pat, pat);
    }
    sql += ' ORDER BY p.created_at DESC';

    const rows = this.db.conn.prepare(sql).all(...params) as unknown as DbProductRow[];
    return rows.map(mapProduct);
  }

  public getProductBySlug(slug: string): ProductRecord {
    const row = this.db.conn
      .prepare('SELECT * FROM products WHERE slug = ? AND active = 1')
      .get(slug) as DbProductRow | undefined;
    if (!row) {
      throw new HttpError(404, 'PRODUCT_NOT_FOUND', `Product '${slug}' not found`);
    }
    return mapProduct(row);
  }

  public createProduct(input: {
    categoryId: string;
    sku: string;
    slug: string;
    title: string;
    description: string;
    priceCents: number;
    stockQty: number;
  }): ProductRecord {
    const id = `prd_${randomUUID()}`;
    const createdAt = new Date().toISOString();
    this.db.conn
      .prepare(
        `INSERT INTO products (id, category_id, sku, slug, title, description, price_cents, stock_qty, active, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, 1, ?)`,
      )
      .run(
        id,
        input.categoryId,
        input.sku,
        input.slug,
        input.title,
        input.description,
        input.priceCents,
        input.stockQty,
        createdAt,
      );
    return this.getProductById(id);
  }

  public updateInventoryOrPrice(
    productId: string,
    patch: { priceCents?: number; stockQty?: number; active?: boolean },
  ): ProductRecord {
    const current = this.getProductById(productId);
    const nextPrice = patch.priceCents ?? current.priceCents;
    const nextStock = patch.stockQty ?? current.stockQty;
    const nextActive = patch.active ?? current.active;

    this.db.conn
      .prepare('UPDATE products SET price_cents = ?, stock_qty = ?, active = ? WHERE id = ?')
      .run(nextPrice, nextStock, nextActive ? 1 : 0, productId);
    return this.getProductById(productId);
  }

  public getProductById(productId: string): ProductRecord {
    const row = this.db.conn
      .prepare('SELECT * FROM products WHERE id = ?')
      .get(productId) as DbProductRow | undefined;
    if (!row) {
      throw new HttpError(404, 'PRODUCT_NOT_FOUND', `Product '${productId}' not found`);
    }
    return mapProduct(row);
  }
}

function mapProduct(r: DbProductRow): ProductRecord {
  return {
    id: r.id,
    categoryId: r.category_id,
    sku: r.sku,
    slug: r.slug,
    title: r.title,
    description: r.description,
    priceCents: r.price_cents,
    stockQty: r.stock_qty,
    active: r.active === 1,
    createdAt: r.created_at,
  };
}
