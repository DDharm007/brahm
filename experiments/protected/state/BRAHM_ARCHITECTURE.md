# BRAHM Architectural Blueprint & Code Intelligence Report

> Generated deterministically by **BRAHM Stage 02 (Code Intelligence & Architecture Mapping Engine)**

## 1. Executive Architecture Overview

| Property | Value |
|---|---|
| **Repository** | `brahm-commerce-demo` |
| **Inferred Pattern** | **REST API / Backend Service** (Confidence: `High`) |
| **Analysis Duration** | `35 ms` |
| **AST Files Parsed** | `26` / `37` total files |
| **Extracted Symbols** | `87` |
| **API Routes Detected** | `20` |
| **Data Models & Schemas** | `10` |
| **Graph Topology** | `155` nodes, `434` edges (DAG: `false`) |

**Architectural Rationale:**
- Explicit HTTP routing layer with 20 endpoint(s)
- Backed by 10 persistence schema(s)/ORM model(s)

## 2. Verification & Epistemological Breakdown

### Verified Static Facts (Deterministic AST & Manifests)
- ✅ Deterministically parsed 26 source files into Tree-sitter concrete syntax trees (0 with syntax error recovery)
- ✅ Extracted 87 unique AST symbols with deterministic BLAKE3 identifiers
- ✅ Observed 108 import declarations (70 internal, 38 external across 12 unique packages)
- ✅ Constructed directed graph containing 155 vertices and 434 edges (DAG: false)
- ✅ Detected 20 explicit API route declarations across source/framework conventions
- ✅ Detected 10 explicit ORM entities, SQL migrations, or schema declarations

### Inferred Relationships & Heuristics
- 🔍 Inferred primary architectural style 'REST API / Backend Service' (Confidence: High)
- 🔍 Classified repository contents into 6 logical architectural layers via path and manifest heuristics
- 🔍 Heuristically inferred 186 function/method call edge(s) from identifier invocation syntax

### Unresolved References (0 total)
_All relative internal imports resolved successfully._

## 3. Application Entry Points

| Kind | File Path | Symbol | Confidence | Description |
|---|---|---|---|---|
| `server_entry` | `src/server.ts` | `-` | `90%` | Web server startup module |

## 4. API Surface & Endpoint Catalog

| Method | Path Pattern | Handler | Framework | Source Location |
|---|---|---|---|---|
| `GET` | `/api/admin/metrics` | `getMetricsHandler` | Express/Fastify | `src/routes/admin.ts:21` |
| `GET` | `/api/admin/audit-logs` | `listAuditLogsHandler` | Express/Fastify | `src/routes/admin.ts:22` |
| `POST` | `/api/auth/register` | `registerHandler` | Express/Fastify | `src/routes/auth.ts:56` |
| `POST` | `/api/auth/login` | `loginHandler` | Express/Fastify | `src/routes/auth.ts:57` |
| `GET` | `/api/auth/me` | `meHandler` | Express/Fastify | `src/routes/auth.ts:58` |
| `GET` | `/api/cart` | `getCartHandler` | Express/Fastify | `src/routes/cart.ts:59` |
| `POST` | `/api/cart/items` | `addCartItemHandler` | Express/Fastify | `src/routes/cart.ts:60` |
| `PATCH` | `/api/cart/items/:productId` | `patchCartItemHandler` | Express/Fastify | `src/routes/cart.ts:61` |
| `DELETE` | `/api/cart/items/:productId` | `removeCartItemHandler` | Express/Fastify | `src/routes/cart.ts:67` |
| `GET` | `/health` | `(_req, res) => {
    res.json({ status: 'ok', service: 'brahm-commerce-api' });
  }` | Express/Fastify | `src/routes/health.ts:5` |
| `POST` | `/api/orders` | `checkoutOrderHandler` | Express/Fastify | `src/routes/orders.ts:60` |
| `GET` | `/api/orders` | `listOrdersHandler` | Express/Fastify | `src/routes/orders.ts:61` |
| `GET` | `/api/orders/:id` | `getOrderHandler` | Express/Fastify | `src/routes/orders.ts:62` |
| `POST` | `/api/orders/:id/cancel` | `cancelOrderHandler` | Express/Fastify | `src/routes/orders.ts:63` |
| `POST` | `/api/payments/intent` | `createIntentHandler` | Express/Fastify | `src/routes/payments.ts:51` |
| `POST` | `/api/payments/webhook` | `webhookHandler` | Express/Fastify | `src/routes/payments.ts:57` |
| `GET` | `/api/products` | `listProductsHandler` | Express/Fastify | `src/routes/products.ts:65` |
| `GET` | `/api/products/:slug` | `getProductHandler` | Express/Fastify | `src/routes/products.ts:66` |
| `POST` | `/api/products` | `createProductHandler` | Express/Fastify | `src/routes/products.ts:67` |
| `PATCH` | `/api/products/:id` | `patchProductHandler` | Express/Fastify | `src/routes/products.ts:74` |

## 5. Data Models & Persistence Layer

| Entity / Artifact | ORM / Technology | Backing Table | File Path |
|---|---|---|---|
| **001_initial_commerce.sql** | SQL Migration | `-` | `db/migrations/001_initial_commerce.sql` |
| **users** | SQL DDL | `users` | `db/schema.sql` |
| **sessions** | SQL DDL | `sessions` | `db/schema.sql` |
| **categories** | SQL DDL | `categories` | `db/schema.sql` |
| **products** | SQL DDL | `products` | `db/schema.sql` |
| **cart_items** | SQL DDL | `cart_items` | `db/schema.sql` |
| **orders** | SQL DDL | `orders` | `db/schema.sql` |
| **order_items** | SQL DDL | `order_items` | `db/schema.sql` |
| **payments** | SQL DDL | `payments` | `db/schema.sql` |
| **audit_logs** | SQL DDL | `audit_logs` | `db/schema.sql` |

## 6. Architectural Layers & Component Distribution

| Architectural Layer | Role | Files | Symbols | Routes |
|---|---|---|---|---|
| **API & Transport Layer** | HTTP route handlers, controllers, and server entrypoints | `8` | `7` | `20` |
| **Configuration & Build Manifests** | Dependency manifests, build configs, and runtime settings | `5` | `0` | `0` |
| **Core Application & Domain Logic** | Primary business logic, services, parsers, and utilities | `15` | `64` | `0` |
| **Documentation** | Architecture specifications, READMEs, and guides | `4` | `0` | `0` |
| **Persistence & Data Layer** | Database schemas, migrations, and ORM entities | `4` | `15` | `0` |
| **Verification & Test Suite** | Automated unit, integration, and verification tests | `1` | `1` | `0` |

## 7. Dependency Topology & Cycles

- **Total Imports:** `108` (`70` internal, `38` external)
- **Circular Dependency Cycles:** `0`

## 8. Architectural Hotspots (Centrality & Blast Radius)

| Component | Kind | Fan-In | Fan-Out | Blast Radius |
|---|---|---|---|---|
| `express` | `external_package` | `12` | `0` | `19` node(s) |
| `errors.ts` | `file` | `11` | `4` | `17` node(s) |
| `runInTransaction` | `method` | `10` | `2` | `31` node(s) |
| `signSessionToken` | `function` | `9` | `4` | `32` node(s) |
| `createProduct` | `method` | `8` | `5` | `14` node(s) |
| `updateInventoryOrPrice` | `method` | `8` | `5` | `14` node(s) |
| `getProductBySlug` | `method` | `8` | `5` | `14` node(s) |
| `verifyWebhookSignature` | `function` | `8` | `4` | `32` node(s) |
| `listProducts` | `method` | `8` | `5` | `14` node(s) |
| `database.ts` | `file` | `7` | `8` | `15` node(s) |

## 9. External Ecosystem Packages

`cors`, `express`, `helmet`, `node:assert`, `node:crypto`, `node:fs`, `node:http`, `node:path`, `node:sqlite`, `node:test`, `node:url`, `zod`

