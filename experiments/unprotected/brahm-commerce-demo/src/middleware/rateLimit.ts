import type { NextFunction, Request, Response } from 'express';
import { HttpError } from './errors.js';

export function createRateLimiter(maxRequests: number, windowMs: number) {
  const hits = new Map<string, number[]>();

  return (req: Request, _res: Response, next: NextFunction): void => {
    const key = req.ip ?? 'unknown';
    const now = Date.now();
    const cutoff = now - windowMs;
    const recent = (hits.get(key) ?? []).filter((ts) => ts >= cutoff);
    if (recent.length >= maxRequests) {
      next(new HttpError(429, 'RATE_LIMITED', 'Too many requests; retry later'));
      return;
    }
    recent.push(now);
    hits.set(key, recent);
    next();
  };
}
