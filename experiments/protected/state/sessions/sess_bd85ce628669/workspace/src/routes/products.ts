import { Router, type NextFunction, type Request, type RequestHandler, type Response } from 'express';
import { z } from 'zod';
import { requireAdmin } from '../middleware/auth.js';
import { validateBody } from '../middleware/validation.js';
import type { CatalogService } from '../services/catalogService.js';

const createProductSchema = z.object({
  categoryId: z.string().min(1),
  sku: z.string().min(2).max(64),
  slug: z.string().min(2).max(120),
  title: z.string().min(2).max(200),
  description: z.string().min(1).max(2000),
  priceCents: z.number().int().nonnegative(),
  stockQty: z.number().int().nonnegative(),
});

const patchProductSchema = z.object({
  priceCents: z.number().int().nonnegative().optional(),
  stockQty: z.number().int().nonnegative().optional(),
  active: z.boolean().optional(),
});

export function createProductsRouter(
  catalog: CatalogService,
  requireAuth: RequestHandler,
): Router {
  const router = Router();

  function listProductsHandler(req: Request, res: Response): void {
    const category = typeof req.query.category === 'string' ? req.query.category : undefined;
    const search = typeof req.query.q === 'string' ? req.query.q : undefined;
    res.json({ items: catalog.listProducts(category, search) });
  }

  function getProductHandler(req: Request, res: Response, next: NextFunction): void {
    try {
      const slug = String(req.params.slug);
      res.json({ product: catalog.getProductBySlug(slug) });
    } catch (err) {
      next(err);
    }
  }

  function createProductHandler(req: Request, res: Response, next: NextFunction): void {
    try {
      const product = catalog.createProduct(req.body as z.infer<typeof createProductSchema>);
      res.status(201).json({ product });
    } catch (err) {
      next(err);
    }
  }

  function patchProductHandler(req: Request, res: Response, next: NextFunction): void {
    try {
      const product = catalog.updateInventoryOrPrice(
        String(req.params.id),
        req.body as z.infer<typeof patchProductSchema>,
      );
      res.json({ product });
    } catch (err) {
      next(err);
    }
  }

  router.get('/api/products', listProductsHandler);
  router.get('/api/products/:slug', getProductHandler);
  router.post(
    '/api/products',
    requireAuth,
    requireAdmin,
    validateBody(createProductSchema),
    createProductHandler,
  );
  router.patch(
    '/api/products/:id',
    requireAuth,
    requireAdmin,
    validateBody(patchProductSchema),
    patchProductHandler,
  );

  return router;
}
