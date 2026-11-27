# BRAHM Commerce API (`brahm-commerce-api`)

A modular, transactional Node.js + TypeScript + Express + SQLite e-commerce backend service created as a clean demonstration target codebase for **BRAHM** anti-reverse-engineering experiments.

## Core Capabilities

- Account registration & login (`scrypt` password hashing + HMAC-SHA256 stateless Bearer tokens)
- Role-based access control (`customer` vs. `admin`)
- Product catalog listing, search, and admin inventory management
- Persistent shopping cart backed by SQLite
- Atomic transactional checkout (`BEGIN IMMEDIATE` ... `COMMIT`), stock reservation, shipping/tax computation (`calculateOrderTotals`), and cancellation stock restoration
- Idempotent payment intent creation and HMAC-SHA256 signed webhook processing
- Admin metrics and immutable audit logging

## Quick Start

```bash
npm install
npm test
npm run build
npm start
```
