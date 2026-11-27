import type { AppConfig } from '../config/env.js';

export interface TotalsBreakdown {
  subtotalCents: number;
  shippingCents: number;
  taxCents: number;
  totalCents: number;
}

export function calculateOrderTotals(
  subtotalCents: number,
  config: Pick<AppConfig, 'taxRateBps' | 'freeShippingThresholdCents' | 'flatShippingCents'>,
): TotalsBreakdown {
  const shippingCents =
    subtotalCents >= config.freeShippingThresholdCents ? 0 : config.flatShippingCents;
  const taxCents = Math.round((subtotalCents * config.taxRateBps) / 10000);
  const totalCents = subtotalCents + shippingCents + taxCents;
  return { subtotalCents, shippingCents, taxCents, totalCents };
}
