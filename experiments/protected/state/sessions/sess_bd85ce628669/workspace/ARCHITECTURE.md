# System Architecture — BRAHM Commerce API

## Layer Overview

1. **HTTP Transport & Middleware (`src/app.ts`, `src/middleware/*`, `src/routes/*`)**
   - `helmet()` security headers, CORS policy, JSON body parsing, sliding-window rate limiter (`200 req/min`).
   - `createAuthMiddleware`: validates `Authorization: Bearer <token>` HMAC-SHA256 tokens (`src/utils/token.ts`).
   - `requireAdmin`: restricts catalog mutations, metrics, and audit log reads to `admin` role.
   - `validateBody`: enforces `zod` schemas on request payloads prior to service invocation.

2. **Domain Services (`src/services/*`)**
   - `AuthService`: user registration, login, `scrypt` verification, and session hash storage.
   - `CatalogService`: category filtering, search, and product CRUD.
   - `CartService`: per-user cart persistence with stock checks.
   - `OrderService`: atomic transaction over cart -> `orders` + `order_items` + stock decrement + `audit_logs`.
   - `PaymentService`: idempotent payment intent issuance and webhook signature validation (`x-commerce-signature`).
   - `AdminService`: aggregate user/product/order/revenue counters and audit feed.

3. **Persistence (`src/db/database.ts`, `db/schema.sql`, `db/migrations/*`)**
   - Uses Node's built-in `node:sqlite` (`DatabaseSync`) with foreign keys enforced.
