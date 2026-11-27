import type { NextFunction, Request, Response } from 'express';
import type { AuthTokenClaims } from '../models/types.js';
import { verifySessionToken } from '../utils/token.js';
import { HttpError } from './errors.js';

export interface AuthenticatedRequest extends Request {
  user?: AuthTokenClaims;
}

export function createAuthMiddleware(tokenSecret: string) {
  return (req: AuthenticatedRequest, _res: Response, next: NextFunction): void => {
    const authHeader = req.headers.authorization;
    if (!authHeader || !authHeader.startsWith('Bearer ')) {
      next(new HttpError(401, 'UNAUTHORIZED', 'Missing Bearer authorization token'));
      return;
    }

    const token = authHeader.slice('Bearer '.length).trim();
    const claims = verifySessionToken(token, tokenSecret);
    if (!claims) {
      next(new HttpError(401, 'INVALID_TOKEN', 'Invalid or expired access token'));
      return;
    }

    req.user = claims;
    next();
  };
}

export function requireAdmin(
  req: AuthenticatedRequest,
  _res: Response,
  next: NextFunction,
): void {
  if (!req.user) {
    next(new HttpError(401, 'UNAUTHORIZED', 'Authentication required'));
    return;
  }
  if (req.user.role !== 'admin') {
    next(new HttpError(403, 'FORBIDDEN', 'Administrator role required'));
    return;
  }
  next();
}
