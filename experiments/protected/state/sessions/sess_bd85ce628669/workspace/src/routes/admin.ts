import { Router, type Request, type RequestHandler, type Response } from 'express';
import { requireAdmin } from '../middleware/auth.js';
import type { AdminService } from '../services/adminService.js';

export function createAdminRouter(
  adminService: AdminService,
  requireAuth: RequestHandler,
): Router {
  const router = Router();

  function getMetricsHandler(_req: Request, res: Response): void {
    res.json({ metrics: adminService.getMetrics() });
  }

  function listAuditLogsHandler(req: Request, res: Response): void {
    const rawLimit = Number(req.query.limit ?? 50);
    const limit = Number.isFinite(rawLimit) ? Math.min(Math.max(rawLimit, 1), 200) : 50;
    res.json({ auditLogs: adminService.listAuditLogs(limit) });
  }

  router.get('/api/admin/metrics', requireAuth, requireAdmin, getMetricsHandler);
  router.get('/api/admin/audit-logs', requireAuth, requireAdmin, listAuditLogsHandler);

  return router;
}
