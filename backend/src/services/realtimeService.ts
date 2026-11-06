import SSEService from './sseService'
import WebSocketService from './websocketService'
import logger from '../utils/logger'

/**
 * Unified Realtime Service.
 * Facade over the SSE and WebSocket services. Both are bridged through the
 * same realtime transport (Redis or RabbitMQ) so they share the same scalable
 * delivery backbone.
 */
class RealtimeService {
  private static instance: RealtimeService
  private sseService = SSEService.getInstance()
  private wsService = WebSocketService.getInstance()

  private constructor () {}

  public static getInstance (): RealtimeService {
    if (!RealtimeService.instance) {
      RealtimeService.instance = new RealtimeService()
    }
    return RealtimeService.instance
  }

  async notifyUser (userId: string | number, event: string, data: unknown): Promise<void> {
    await Promise.all([
      this.sseService.sendToUser(userId, event, data),
      this.wsService.sendToUser(userId, event, data)
    ])
    logger.debug('Realtime notification sent to user', { userId, event })
  }

  async broadcastToAll (event: string, data: unknown): Promise<void> {
    await Promise.all([
      this.sseService.broadcast(event, data),
      this.wsService.broadcast(event, data)
    ])
    logger.debug('Realtime broadcast sent', { event })
  }

  async sendToChannel (channel: string, event: string, data: unknown): Promise<void> {
    await Promise.all([
      this.sseService.sendToChannel(channel, event, data),
      this.wsService.sendToChannel(channel, event, data)
    ])
    logger.debug('Realtime channel message sent', { channel, event })
  }

  async sendSSE (userId: string | number, event: string, data: unknown): Promise<void> {
    await this.sseService.sendToUser(userId, event, data)
  }

  async sendWebSocket (userId: string | number, event: string, data: unknown): Promise<void> {
    await this.wsService.sendToUser(userId, event, data)
  }

  async getStats (): Promise<{
    sse: {totalUsers: number; totalConnections: number}
    websocket: {totalUsers: number; totalConnections: number; onlineUsers: string[]}
  }> {
    const [sseStats, wsStats] = await Promise.all([
      Promise.resolve(this.sseService.getStats()),
      this.wsService.getStats()
    ])

    return {
      sse: sseStats,
      websocket: wsStats
    }
  }

  isUserOnline (userId: string | number): boolean {
    return this.wsService.isUserOnline(userId)
  }
}

export default RealtimeService