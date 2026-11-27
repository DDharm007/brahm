import { Router, type NextFunction, type RequestHandler, type Response } from 'express';
import { z } from 'zod';
import type { AuthenticatedRequest } from '../middleware/auth.js';
import { validateBody } from '../middleware/validation.js';
import type { OrderService } from '../services/orderService.js';

const createOrderSchema = z.object({
  shippingAddress: z.object({
    line1: z.string().min(3),
    city: z.string().min(2),
    postalCode: z.string().min(3),
    country: z.string().length(2),
  }),
});

export function createOrdersRouter(
  orderService: OrderService,
  requireAuth: RequestHandler,
): Router {
  const router = Router();

  function checkoutOrderHandler(
    req: AuthenticatedRequest,
    res: Response,
    next: NextFunction,
  ): void {
    try {
      const body = req.body as z.infer<typeof createOrderSchema>;
      const order = orderService.checkoutFromCart(req.user!.sub, body.shippingAddress);
      res.status(201).json({ order });
    } catch (err) {
      next(err);
    }
  }

  function listOrdersHandler(req: AuthenticatedRequest, res: Response): void {
    res.json({ orders: orderService.listOrdersForUser(req.user!.sub) });
  }

  function getOrderHandler(req: AuthenticatedRequest, res: Response, next: NextFunction): void {
    try {
      const isAdmin = req.user!.role === 'admin';
      const order = orderService.getOrder(String(req.params.id), req.user!.sub, isAdmin);
      res.json({ order });
    } catch (err) {
      next(err);
    }
  }

  function cancelOrderHandler(req: AuthenticatedRequest, res: Response, next: NextFunction): void {
    try {
      const isAdmin = req.user!.role === 'admin';
      const order = orderService.cancelOrder(String(req.params.id), req.user!.sub, isAdmin);
      res.json({ order });
    } catch (err) {
      next(err);
    }
  }

  router.post('/api/orders', requireAuth, validateBody(createOrderSchema), checkoutOrderHandler);
  router.get('/api/orders', requireAuth, listOrdersHandler);
  router.get('/api/orders/:id', requireAuth, getOrderHandler);
  router.post('/api/orders/:id/cancel', requireAuth, cancelOrderHandler);

  return router;
}
