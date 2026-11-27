import assert from 'node:assert/strict';
import http from 'node:http';
import path from 'node:path';
import { after, before, describe, it } from 'node:test';
import { fileURLToPath } from 'node:url';
import { createCommerceApp } from '../src/app.js';
import type { AppConfig } from '../src/config/env.js';
import { computeWebhookSignature } from '../src/utils/token.js';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const projectRoot = path.resolve(__dirname, '..', '..');

const testConfig: AppConfig = {
  port: 0,
  nodeEnv: 'test',
  sqliteDbPath: ':memory:',
  tokenSigningSecret: 'test-only-token-signing-secret-xyz',
  paymentWebhookSecret: 'test-only-webhook-secret-xyz',
  corsOrigin: 'http://localhost:3000',
  taxRateBps: 800,
  freeShippingThresholdCents: 10000,
  flatShippingCents: 1000,
};

async function apiJson<T = unknown>(
  baseUrl: string,
  route: string,
  options: { method?: string; token?: string; headers?: Record<string, string>; body?: unknown } = {},
): Promise<{ status: number; data: T }> {
  const headers: Record<string, string> = {
    'content-type': 'application/json',
    ...options.headers,
  };
  if (options.token) {
    headers.authorization = `Bearer ${options.token}`;
  }
  const resp = await fetch(`${baseUrl}${route}`, {
    method: options.method ?? 'GET',
    headers,
    body: options.body === undefined ? undefined : JSON.stringify(options.body),
  });
  const data = (await resp.json()) as T;
  return { status: resp.status, data };
}

describe('BRAHM Commerce API Integration Tests', () => {
  let server: http.Server;
  let baseUrl: string;
  let adminToken: string;
  let customerToken: string;
  let productId: string;

  before(async () => {
    const { app, services } = createCommerceApp(testConfig, projectRoot);
    const adminReg = services.auth.registerUser({
      email: 'admin@brahm-demo.local',
      fullName: 'Demo Admin',
      password: 'AdminPassword123!',
      role: 'admin',
    });
    adminToken = adminReg.token;

    await new Promise<void>((resolve) => {
      server = app.listen(0, () => resolve());
    });
    const addr = server.address() as { port: number };
    baseUrl = `http://127.0.0.1:${addr.port}`;
  });

  after(async () => {
    await new Promise<void>((resolve, reject) => {
      server.close((err) => (err ? reject(err) : resolve()));
    });
  });

  it('serves health check and registers customer', async () => {
    const health = await apiJson<{ status: string }>(baseUrl, '/health');
    assert.equal(health.status, 200);
    assert.equal(health.data.status, 'ok');

    const reg = await apiJson<{ user: { role: string }; token: string }>(
      baseUrl,
      '/api/auth/register',
      {
        method: 'POST',
        body: {
          email: 'alice@example.com',
          fullName: 'Alice Example',
          password: 'StrongPassword99!',
        },
      },
    );
    assert.equal(reg.status, 201);
    assert.equal(reg.data.user.role, 'customer');
    customerToken = reg.data.token;
  });

  it('enforces admin RBAC on product creation', async () => {
    const forbidden = await apiJson(baseUrl, '/api/products', {
      method: 'POST',
      token: customerToken,
      body: {
        categoryId: 'cat_hw',
        sku: 'KEY-01',
        slug: 'yubikey-demo',
        title: 'Security Key',
        description: 'FIDO2 device',
        priceCents: 4500,
        stockQty: 10,
      },
    });
    assert.equal(forbidden.status, 403);

    const created = await apiJson<{ product: { id: string } }>(baseUrl, '/api/products', {
      method: 'POST',
      token: adminToken,
      body: {
        categoryId: 'cat_hw',
        sku: 'KEY-01',
        slug: 'fido2-key-pro',
        title: 'FIDO2 Key Pro',
        description: 'USB-C hardware authenticator',
        priceCents: 6000,
        stockQty: 5,
      },
    });
    assert.equal(created.status, 201);
    productId = created.data.product.id;
  });

  it('completes transactional cart checkout, payment webhook, and admin metrics', async () => {
    const added = await apiJson<{ subtotalCents: number }>(baseUrl, '/api/cart/items', {
      method: 'POST',
      token: customerToken,
      body: { productId, quantity: 2 },
    });
    assert.equal(added.status, 201);
    assert.equal(added.data.subtotalCents, 12000); // Free shipping threshold reached

    const orderRes = await apiJson<{
      order: { id: string; status: string; taxCents: number; shippingCents: number; totalCents: number };
    }>(baseUrl, '/api/orders', {
      method: 'POST',
      token: customerToken,
      body: {
        shippingAddress: {
          line1: '100 Security Way',
          city: 'Austin',
          postalCode: '78701',
          country: 'US',
        },
      },
    });
    assert.equal(orderRes.status, 201);
    assert.equal(orderRes.data.order.status, 'pending_payment');
    assert.equal(orderRes.data.order.shippingCents, 0);
    assert.equal(orderRes.data.order.taxCents, 960);
    assert.equal(orderRes.data.order.totalCents, 12960);

    const intent = await apiJson<{ payment: { providerIntentId: string; status: string } }>(
      baseUrl,
      '/api/payments/intent',
      {
        method: 'POST',
        token: customerToken,
        body: { orderId: orderRes.data.order.id, idempotencyKey: 'idem_001' },
      },
    );
    assert.equal(intent.status, 201);
    assert.equal(intent.data.payment.status, 'requires_confirmation');

    const webhookPayload = {
      providerIntentId: intent.data.payment.providerIntentId,
      status: 'succeeded',
    };
    const sig = computeWebhookSignature(
      JSON.stringify(webhookPayload),
      testConfig.paymentWebhookSecret,
    );
    const webhookRes = await apiJson<{ payment: { status: string } }>(
      baseUrl,
      '/api/payments/webhook',
      {
        method: 'POST',
        headers: { 'x-commerce-signature': sig },
        body: webhookPayload,
      },
    );
    assert.equal(webhookRes.status, 200);
    assert.equal(webhookRes.data.payment.status, 'succeeded');

    const metrics = await apiJson<{ metrics: { paidRevenueCents: number; totalOrders: number } }>(
      baseUrl,
      '/api/admin/metrics',
      { token: adminToken },
    );
    assert.equal(metrics.status, 200);
    assert.equal(metrics.data.metrics.totalOrders, 1);
    assert.equal(metrics.data.metrics.paidRevenueCents, 12960);
  });
});
