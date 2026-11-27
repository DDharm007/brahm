import { Router, type NextFunction, type Request, type RequestHandler, type Response } from 'express';
import { z } from 'zod';
import type { AuthenticatedRequest } from '../middleware/auth.js';
import { HttpError } from '../middleware/errors.js';
import { validateBody } from '../middleware/validation.js';
import type { PaymentService } from '../services/paymentService.js';

const createIntentSchema = z.object({
  orderId: z.string().min(1),
  idempotencyKey: z.string().min(4).max(128),
});

export function createPaymentsRouter(
  paymentService: PaymentService,
  requireAuth: RequestHandler,
): Router {
  const router = Router();

  function createIntentHandler(
    req: AuthenticatedRequest,
    res: Response,
    next: NextFunction,
  ): void {
    try {
      const body = req.body as z.infer<typeof createIntentSchema>;
      const payment = paymentService.createPaymentIntent(
        body.orderId,
        req.user!.sub,
        body.idempotencyKey,
      );
      res.status(201).json({ payment });
    } catch (err) {
      next(err);
    }
  }

  function webhookHandler(req: Request, res: Response, next: NextFunction): void {
    try {
      const sig = req.headers['x-commerce-signature'];
      if (typeof sig !== 'string' || sig.length === 0) {
        throw new HttpError(401, 'MISSING_SIGNATURE', 'Missing webhook signature header');
      }
      const rawBody = JSON.stringify(req.body);
      const payment = paymentService.processWebhook(rawBody, sig);
      res.json({ payment });
    } catch (err) {
      next(err);
    }
  }

  router.post(
    '/api/payments/intent',
    requireAuth,
    validateBody(createIntentSchema),
    createIntentHandler,
  );
  router.post('/api/payments/webhook', webhookHandler);

  return router;
}
