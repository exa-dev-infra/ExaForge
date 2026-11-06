import type { Request, Response } from 'express'
import { Router } from 'express'
import SSEService from '../../services/sseService'
import { authenticate } from '../../middleware/auth'
import ErrorHandler from '../../middleware/errorHandler'
import { ApiResponse } from '../../types/response'
import { validate } from '../../utils/validator'
import { sseTestSchema } from './validation'

const router = Router()
const sseService = SSEService.getInstance()

/**
 * SSE endpoint - untuk menerima real-time updates.
 * Client harus terautentikasi.
 */
router.get(
  '/stream',
  authenticate,
  ErrorHandler.catchAsync(async (req: Request, res: Response) => {
    const userId = req.user?.userId

    if (!userId) {
      res.status(401).json(ApiResponse.unauthorized())
      return
    }

    // Add client to SSE service; the connection stays open until disconnect.
    sseService.addClient(userId, res)
  })
)

/**
 * Get SSE stats (monitoring)
 */
router.get(
  '/stats',
  authenticate,
  ErrorHandler.catchAsync(async (_req: Request, res: Response) => {
    const stats = sseService.getStats()
    res.json(ApiResponse.success(stats))
  })
)

/**
 * Test endpoint - push a JSON payload to the authenticated user's streams
 */
router.post(
  '/test',
  authenticate,
  validate(sseTestSchema),
  ErrorHandler.catchAsync(async (req: Request, res: Response) => {
    const userId = req.user?.userId
    const { message, data } = req.validated?.body ?? {}

    await sseService.sendToUser(userId!, 'test', {
      message: message || 'Test message from SSE',
      data,
      timestamp: new Date().toISOString()
    })

    res.json(ApiResponse.success({ sent: true }))
  })
)

export default router