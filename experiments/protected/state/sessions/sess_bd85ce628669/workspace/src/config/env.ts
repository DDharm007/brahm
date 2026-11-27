export interface AppConfig {
  port: number;
  nodeEnv: string;
  sqliteDbPath: string;
  tokenSigningSecret: string;
  paymentWebhookSecret: string;
  corsOrigin: string;
  taxRateBps: number;
  freeShippingThresholdCents: number;
  flatShippingCents: number;
}

export function loadConfig(): AppConfig {
  return {
    port: Number(process.env.PORT ?? 3100),
    nodeEnv: process.env.NODE_ENV ?? 'development',
    sqliteDbPath: process.env.SQLITE_DB_PATH ?? './data/commerce.sqlite',
    tokenSigningSecret:
      process.env.TOKEN_SIGNING_SECRET ?? 'dev-only-ephemeral-hmac-key-change-in-prod',
    paymentWebhookSecret:
      process.env.PAYMENT_WEBHOOK_SECRET ?? 'dev-only-webhook-secret-change-in-prod',
    corsOrigin: process.env.CORS_ORIGIN ?? 'http://localhost:3000',
    taxRateBps: Number(process.env.TAX_RATE_BPS ?? 825),
    freeShippingThresholdCents: Number(process.env.FREE_SHIPPING_THRESHOLD_CENTS ?? 10000),
    flatShippingCents: Number(process.env.FLAT_SHIPPING_CENTS ?? 995),
  };
}
