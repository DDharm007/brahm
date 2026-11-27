import path from 'node:path';
import cors from 'cors';
import express, { type Express } from 'express';
import helmet from 'helmet';
import type { AppConfig } from './config/env.js';
import { CommerceDatabase } from './db/database.js';
import { createAuthMiddleware } from './middleware/auth.js';
import { errorHandler } from './middleware/errors.js';
import { createRateLimiter } from './middleware/rateLimit.js';
import { createAdminRouter } from './routes/admin.js';
import { createAuthRouter } from './routes/auth.js';
import { createCartRouter } from './routes/cart.js';
import { createHealthRouter } from './routes/health.js';
import { createOrdersRouter } from './routes/orders.js';
import { createPaymentsRouter } from './routes/payments.js';
import { createProductsRouter } from './routes/products.js';
import { AdminService } from './services/adminService.js';
import { AuthService } from './services/authService.js';
import { CartService } from './services/cartService.js';
import { CatalogService } from './services/catalogService.js';
import { OrderService } from './services/orderService.js';
import { PaymentService } from './services/paymentService.js';

export interface AppServices {
  db: CommerceDatabase;
  auth: AuthService;
  catalog: CatalogService;
  cart: CartService;
  orders: OrderService;
  payments: PaymentService;
  admin: AdminService;
}

export function createCommerceApp(
  config: AppConfig,
  projectRootDir: string,
): { app: Express; services: AppServices } {
  const db = new CommerceDatabase(config.sqliteDbPath);
  db.initializeSchema(
    path.join(projectRootDir, 'db', 'schema.sql'),
    path.join(projectRootDir, 'db', 'migrations', '001_initial_commerce.sql'),
  );

  const auth = new AuthService(db, config.tokenSigningSecret);
  const catalog = new CatalogService(db);
  const cart = new CartService(db);
  const orders = new OrderService(db, config);
  const payments = new PaymentService(db, config.paymentWebhookSecret);
  const admin = new AdminService(db);

  const app = express();
  app.use(helmet());
  app.use(cors({ origin: config.corsOrigin }));
  app.use(express.json({ limit: '1mb' }));
  app.use(createRateLimiter(200, 60_000));

  const requireAuth = createAuthMiddleware(config.tokenSigningSecret);
  app.use(createHealthRouter());
  app.use(createAuthRouter(auth, requireAuth));
  app.use(createProductsRouter(catalog, requireAuth));
  app.use(createCartRouter(cart, requireAuth));
  app.use(createOrdersRouter(orders, requireAuth));
  app.use(createPaymentsRouter(payments, requireAuth));
  app.use(createAdminRouter(admin, requireAuth));

  app.use(errorHandler);

  return {
    app,
    services: { db, auth, catalog, cart, orders, payments, admin },
  };
}
