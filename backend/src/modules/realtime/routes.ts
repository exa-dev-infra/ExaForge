import type { Request, Response } from 'express'
import { Router } from 'express'
import RealtimeService from '../../services/realtimeService'
import WebSocketService from '../../services/websocketService'
import { authenticate } from '../../middleware/auth'
import ErrorHandler from '../../middleware/errorHandler'
import { ApiResponse } from '../../types/response'
import { validate } from '../../utils/validator'
import { broadcastSchema, channelSchema, disconnectSchema, notifyUserSchema, onlineUserSchema } from './validation'

const router = Router()
const realtimeService = RealtimeService.getInstance()
const wsService = WebSocketService.getInstance()

/**
 * Send notification to specific user
 */
router.post(
  '/notify/:userId',
  authenticate,
  validate(notifyUserSchema),
  ErrorHandler.catchAsync(async (req: Request, res: Response) => {
    const userId = String(req.params.userId)
    const { event, data } = req.validated?.body ?? {}

    await realtimeService.notifyUser(userId, String(event), data)

    res.json(ApiResponse.success({
      sent: true,
      userId,
      event
    }))
  })
)

/**
 * Broadcast to all connected users
 */
router.post(
  '/broadcast',
  authenticate,
  validate(broadcastSchema),
  ErrorHandler.catchAsync(async (req: Request, res: Response) => {
    const { event, data } = req.validated?.body ?? {}

    await realtimeService.broadcastToAll(String(event), data)

    res.json(ApiResponse.success({
      broadcasted: true,
      event
    }))
  })
)

/**
 * Send to specific channel/room
 */
router.post(
  '/channel/:channel',
  authenticate,
  validate(channelSchema),
  ErrorHandler.catchAsync(async (req: Request, res: Response) => {
    const channel = String(req.params.channel)
    const { event, data } = req.validated?.body ?? {}

    await realtimeService.sendToChannel(channel, String(event), data)

    res.json(ApiResponse.success({
      sent: true,
      channel,
      event
    }))
  })
)

/**
 * Get realtime statistics
 */
router.get(
  '/stats',
  authenticate,
  ErrorHandler.catchAsync(async (_req: Request, res: Response) => {
    const stats = await realtimeService.getStats()
    res.json(ApiResponse.success(stats))
  })
)

/**
 * Check if user is online
 */
router.get(
  '/online/:userId',
  authenticate,
  validate(onlineUserSchema),
  ErrorHandler.catchAsync(async (req: Request, res: Response) => {
    const userId = String(req.params.userId)
    const isOnline = realtimeService.isUserOnline(userId)

    res.json(ApiResponse.success({
      userId,
      isOnline
    }))
  })
)

/**
 * Disconnect specific user (admin only)
 */
router.post(
  '/disconnect/:userId',
  authenticate,
  validate(disconnectSchema),
  ErrorHandler.catchAsync(async (req: Request, res: Response) => {
    const userId = String(req.params.userId)
    const reason = req.validated?.body?.reason as string | undefined

    await wsService.disconnectUser(userId, reason)

    res.json(ApiResponse.success({
      disconnected: true,
      userId
    }))
  })
)

export default router