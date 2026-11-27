import { Router, type NextFunction, type RequestHandler, type Response } from 'express';
import { z } from 'zod';
import type { AuthenticatedRequest } from '../middleware/auth.js';
import { HttpError } from '../middleware/errors.js';
import { validateBody } from '../middleware/validation.js';
import type { AuthService } from '../services/authService.js';

const registerSchema = z.object({
  email: z.string().email(),
  fullName: z.string().min(2).max(120),
  password: z.string().min(8).max(128),
});

const loginSchema = z.object({
  email: z.string().email(),
  password: z.string().min(1),
});

export function createAuthRouter(
  authService: AuthService,
  requireAuth: RequestHandler,
): Router {
  const router = Router();

  function registerHandler(req: AuthenticatedRequest, res: Response, next: NextFunction): void {
    try {
      const body = req.body as z.infer<typeof registerSchema>;
      const result = authService.registerUser(body);
      res.status(201).json(result);
    } catch (err) {
      next(err);
    }
  }

  function loginHandler(req: AuthenticatedRequest, res: Response, next: NextFunction): void {
    try {
      const body = req.body as z.infer<typeof loginSchema>;
      const result = authService.loginUser(body.email, body.password);
      res.json(result);
    } catch (err) {
      next(err);
    }
  }

  function meHandler(req: AuthenticatedRequest, res: Response, next: NextFunction): void {
    try {
      if (!req.user) {
        throw new HttpError(401, 'UNAUTHORIZED', 'Authentication required');
      }
      res.json({ user: authService.getUserById(req.user.sub) });
    } catch (err) {
      next(err);
    }
  }

  router.post('/api/auth/register', validateBody(registerSchema), registerHandler);
  router.post('/api/auth/login', validateBody(loginSchema), loginHandler);
  router.get('/api/auth/me', requireAuth, meHandler);

  return router;
}
