import { createHash, createHmac, randomUUID, timingSafeEqual } from 'node:crypto';
import type { AuthTokenClaims, UserRole } from '../models/types.js';

export function signSessionToken(
  userId: string,
  email: string,
  role: UserRole,
  secret: string,
  ttlSeconds = 3600,
): { token: string; claims: AuthTokenClaims; tokenHash: string } {
  const claims: AuthTokenClaims = {
    sub: userId,
    email,
    role,
    jti: randomUUID(),
    exp: Math.floor(Date.now() / 1000) + ttlSeconds,
  };
  const payload = Buffer.from(JSON.stringify(claims), 'utf8').toString('base64url');
  const sig = createHmac('sha256', secret).update(payload).digest('base64url');
  const token = `${payload}.${sig}`;
  return { token, claims, tokenHash: hashToken(token) };
}

export function verifySessionToken(token: string, secret: string): AuthTokenClaims | null {
  const parts = token.split('.');
  if (parts.length !== 2) {
    return null;
  }
  const [payload, sig] = parts;
  const expected = createHmac('sha256', secret).update(payload).digest('base64url');
  const a = Buffer.from(sig);
  const b = Buffer.from(expected);
  if (a.length !== b.length || !timingSafeEqual(a, b)) {
    return null;
  }
  const parsed = JSON.parse(Buffer.from(payload, 'base64url').toString('utf8')) as AuthTokenClaims;
  if (parsed.exp <= Math.floor(Date.now() / 1000)) {
    return null;
  }
  return parsed;
}

export function hashToken(token: string): string {
  return createHash('sha256').update(token).digest('hex');
}

export function computeWebhookSignature(payload: string, secret: string): string {
  return createHmac('sha256', secret).update(payload).digest('hex');
}

export function verifyWebhookSignature(
  payload: string,
  providedHex: string,
  secret: string,
): boolean {
  const expectedHex = computeWebhookSignature(payload, secret);
  const a = Buffer.from(providedHex, 'hex');
  const b = Buffer.from(expectedHex, 'hex');
  return a.length > 0 && a.length === b.length && timingSafeEqual(a, b);
}
