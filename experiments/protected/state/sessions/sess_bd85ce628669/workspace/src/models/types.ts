export type UserRole = 'customer' | 'admin';

export type OrderStatus =
  | 'pending_payment'
  | 'paid'
  | 'fulfilled'
  | 'cancelled'
  | 'refunded';

export type PaymentStatus =
  | 'requires_confirmation'
  | 'succeeded'
  | 'failed'
  | 'refunded';

export interface UserRecord {
  id: string;
  email: string;
  fullName: string;
  role: UserRole;
  createdAt: string;
}

export interface ProductRecord {
  id: string;
  categoryId: string;
  sku: string;
  slug: string;
  title: string;
  description: string;
  priceCents: number;
  stockQty: number;
  active: boolean;
  createdAt: string;
}

export interface CartItemView {
  productId: string;
  sku: string;
  title: string;
  unitPriceCents: number;
  quantity: number;
  lineTotalCents: number;
}

export interface ShippingAddress {
  line1: string;
  city: string;
  postalCode: string;
  country: string;
}

export interface OrderRecord {
  id: string;
  userId: string;
  status: OrderStatus;
  subtotalCents: number;
  shippingCents: number;
  taxCents: number;
  totalCents: number;
  shippingAddress: ShippingAddress;
  createdAt: string;
  updatedAt: string;
}

export interface PaymentRecord {
  id: string;
  orderId: string;
  providerIntentId: string;
  status: PaymentStatus;
  amountCents: number;
  idempotencyKey: string;
  processedAt: string | null;
}

export interface AuthTokenClaims {
  sub: string;
  email: string;
  role: UserRole;
  jti: string;
  exp: number;
}
