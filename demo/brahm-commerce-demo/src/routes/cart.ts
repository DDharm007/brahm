import { Router, type NextFunction, type RequestHandler, type Response } from 'express';
import { z } from 'zod';
import type { AuthenticatedRequest } from '../middleware/auth.js';
import { validateBody } from '../middleware/validation.js';
import type { CartService } from '../services/cartService.js';

const addCartSchema = z.object({
  productId: z.string().min(1),
  quantity: z.number().int().positive(),
});

const patchCartSchema = z.object({
  quantity: z.number().int().nonnegative(),
});

export function createCartRouter(
  cartService: CartService,
  requireAuth: RequestHandler,
): Router {
  const router = Router();

  function getCartHandler(req: AuthenticatedRequest, res: Response): void {
    res.json(cartService.getCart(req.user!.sub));
  }

  function addCartItemHandler(
    req: AuthenticatedRequest,
    res: Response,
    next: NextFunction,
  ): void {
    try {
      const body = req.body as z.infer<typeof addCartSchema>;
      cartService.addItem(req.user!.sub, body.productId, body.quantity);
      res.status(201).json(cartService.getCart(req.user!.sub));
    } catch (err) {
      next(err);
    }
  }

  function patchCartItemHandler(
    req: AuthenticatedRequest,
    res: Response,
    next: NextFunction,
  ): void {
    try {
      const body = req.body as z.infer<typeof patchCartSchema>;
      cartService.setQuantity(req.user!.sub, String(req.params.productId), body.quantity);
      res.json(cartService.getCart(req.user!.sub));
    } catch (err) {
      next(err);
    }
  }

  function removeCartItemHandler(req: AuthenticatedRequest, res: Response): void {
    cartService.removeItem(req.user!.sub, String(req.params.productId));
    res.json(cartService.getCart(req.user!.sub));
  }

  router.get('/api/cart', requireAuth, getCartHandler);
  router.post('/api/cart/items', requireAuth, validateBody(addCartSchema), addCartItemHandler);
  router.patch(
    '/api/cart/items/:productId',
    requireAuth,
    validateBody(patchCartSchema),
    patchCartItemHandler,
  );
  router.delete('/api/cart/items/:productId', requireAuth, removeCartItemHandler);

  return router;
}
