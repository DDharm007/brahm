# API Documentation — BRAHM Commerce API

| Method | Route | Auth | Description |
| :--- | :--- | :--- | :--- |
| `GET` | `/health` | Public | Health check returning `{ status: "ok", service: "brahm-commerce-api" }`. |
| `POST` | `/api/auth/register` | Public | Register a new customer account; returns `{ user, token }`. |
| `POST` | `/api/auth/login` | Public | Authenticate user credentials; returns `{ user, token }`. |
| `GET` | `/api/auth/me` | Bearer | Retrieve authenticated user profile. |
| `GET` | `/api/products` | Public | List active products (`?category=<slug>&q=<search>`). |
| `GET` | `/api/products/:slug` | Public | Fetch product details by slug. |
| `POST` | `/api/products` | Admin | Create a new catalog product. |
| `PATCH` | `/api/products/:id` | Admin | Update product price, stock quantity, or active status. |
| `GET` | `/api/cart` | Bearer | Get current user's cart items and subtotal. |
| `POST` | `/api/cart/items` | Bearer | Add item (`{ productId, quantity }`) to cart. |
| `PATCH` | `/api/cart/items/:productId` | Bearer | Set exact item quantity (`0` removes item). |
| `DELETE` | `/api/cart/items/:productId` | Bearer | Remove product from cart. |
| `POST` | `/api/orders` | Bearer | Transactional checkout from cart with shipping address. |
| `GET` | `/api/orders` | Bearer | List authenticated user's orders. |
| `GET` | `/api/orders/:id` | Bearer/Admin | Retrieve order details (owner or admin). |
| `POST` | `/api/orders/:id/cancel` | Bearer/Admin | Cancel a `pending_payment` order and restore product stock. |
| `POST` | `/api/payments/intent` | Bearer | Idempotent payment intent creation (`idempotencyKey`). |
| `POST` | `/api/payments/webhook` | Webhook Sig | Signed provider callback (`x-commerce-signature` HMAC-SHA256). |
| `GET` | `/api/admin/metrics` | Admin | Aggregate counts and paid revenue totals. |
| `GET` | `/api/admin/audit-logs` | Admin | Recent immutable audit log entries (`?limit=50`). |
