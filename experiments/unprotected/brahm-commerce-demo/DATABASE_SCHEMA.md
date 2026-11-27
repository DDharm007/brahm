# Database Schema — BRAHM Commerce API

Defined canonically in `db/schema.sql` with initial category seeds in `db/migrations/001_initial_commerce.sql`:

| Table | Purpose | Key Constraints |
| :--- | :--- | :--- |
| `users` | Account credentials & RBAC role | `email` UNIQUE, `role IN ('customer', 'admin')` |
| `sessions` | Issued session SHA-256 digests | `user_id -> users(id)`, `token_hash` UNIQUE |
| `categories` | Product taxonomy | `slug` UNIQUE |
| `products` | Inventory & integer cent pricing | `category_id -> categories(id)`, `sku` & `slug` UNIQUE |
| `cart_items` | Persistent shopping cart | `UNIQUE(user_id, product_id)` |
| `orders` | Order totals & status machine | `user_id -> users(id)`, 5-state `status` CHECK |
| `order_items` | Line-item price snapshots | `order_id -> orders(id)`, `product_id -> products(id)` |
| `payments` | Idempotent payment intents & outcomes | `order_id -> orders(id)`, `provider_intent_id` & `idempotency_key` UNIQUE |
| `audit_logs` | Order and administrative audit trail | Indexed by `created_at DESC` |
