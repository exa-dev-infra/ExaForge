import type { Response } from 'express'
import type { RealtimeBridge } from './realtime'
import { createBridge } from './realtime'
import logger from '../utils/logger'

const KEEP_ALIVE_INTERVAL_MS = 30000

/**
 * SSE Service - scalable Server-Sent Events.
 * Messages are fanned out through a realtime bridge (Redis or RabbitMQ), so
 * any instance can push events to clients that are connected to another one.
 */
class SSEService {
  private static instance: SSEService
  private clients: Map<string, Set<Response>> = new Map()
  private channelSubscribers = new Set<string>()
  private bridge: RealtimeBridge
  private initialized = false

  private constructor () {
    this.bridge = createBridge()
  }

  public static getInstance (): SSEService {
    if (!SSEService.instance) {
      SSEService.instance = new SSEService()
    }
    return SSEService.instance
  }

  async initialize (): Promise<void> {
    if (this.initialized) {return}

    await this.bridge.init()
    await this.bridge.subscribe('sse:*', this.handleBridgeMessage.bind(this))

    this.initialized = true
    logger.info('SSE service initialized with realtime bridge')
  }

  addClient (userId: string | number, res: Response): void {
    const key = String(userId)

    if (!this.clients.has(key)) {
      this.clients.set(key, new Set())
    }

    const userClients = this.clients.get(key)!
    userClients.add(res)

    res.writeHead(200, {
      'Content-Type': 'text/event-stream',
      'Cache-Control': 'no-cache',
      Connection: 'keep-alive',
      'X-Accel-Buffering': 'no'
    })

    this.sendToClient(res, 'connected', {
      message: 'Connected to SSE',
      timestamp: new Date().toISOString()
    })

    const keepAliveInterval = setInterval(() => {
      if (res.writable) {
        res.write(': keep-alive\n\n')
      } else {
        clearInterval(keepAliveInterval)
      }
    }, KEEP_ALIVE_INTERVAL_MS)

    res.on('close', () => {
      clearInterval(keepAliveInterval)
      this.removeClient(key, res)
    })

    logger.debug('SSE client connected', { userId: key })
  }

  removeClient (userId: string | number, res: Response): void {
    const key = String(userId)
    const userClients = this.clients.get(key)

    if (!userClients) {return}

    userClients.delete(res)

    if (userClients.size === 0) {
      this.clients.delete(key)
    }

    logger.debug('SSE client disconnected', { userId: key })
  }

  async sendToUser (userId: string | number, event: string, data: unknown): Promise<void> {
    const message = JSON.stringify({ event, data, userId: String(userId) })
    await this.bridge.publish(`sse:user:${String(userId)}`, message)
  }

  async broadcast (event: string, data: unknown): Promise<void> {
    const message = JSON.stringify({ event, data })
    await this.bridge.publish('sse:broadcast', message)
  }

  async sendToChannel (channel: string, event: string, data: unknown): Promise<void> {
    const message = JSON.stringify({ event, data, channel })
    await this.bridge.publish(`sse:channel:${channel}`, message)
  }

  subscribeToChannel (userId: string | number, channel: string): void {
    this.channelSubscribers.add(`sub:${channel}:${String(userId)}`)
  }

  unsubscribeFromChannel (userId: string | number, channel: string): void {
    this.channelSubscribers.delete(`sub:${channel}:${String(userId)}`)
  }

  private handleBridgeMessage = (channelName: string, message: string): void => {
    try {
      const parsed = JSON.parse(message) as {event: string; data: unknown; userId: string}
      const channelPrefix = 'sse:channel:'

      if (channelName === 'sse:broadcast') {
        this.broadcastLocal(parsed.event, parsed.data)
      } else if (channelName.startsWith('sse:user:')) {
        this.sendToUserLocal(parsed.userId, parsed.event, parsed.data)
      } else if (channelName.startsWith(channelPrefix)) {
        this.sendToChannelLocal(channelName.slice(channelPrefix.length), parsed.event, parsed.data)
      }
    } catch (error) {
      logger.error('SSE bridge message handling failed', { error })
    }
  }

  private sendToUserLocal (userId: string, event: string, data: unknown): void {
    const userClients = this.clients.get(userId)

    if (!userClients) {return}

    userClients.forEach((res) => this.sendToClient(res, event, data))
  }

  private broadcastLocal (event: string, data: unknown): void {
    this.clients.forEach((userClients) => {
      userClients.forEach((res) => this.sendToClient(res, event, data))
    })
  }

  private sendToChannelLocal (channel: string, event: string, data: unknown): void {
    const targets = new Set<Response>()

    this.clients.forEach((userClients, userId) => {
      if (this.channelSubscribers.has(`sub:${channel}:${userId}`)) {
        userClients.forEach((res) => targets.add(res))
      }
    })

    if (targets.size === 0) {return}

    targets.forEach((res) => this.sendToClient(res, event, data))
  }

  private sendToClient (res: Response, event: string, data: unknown): void {
    if (!res.writable) {return}

    try {
      const payload = JSON.stringify(data)
      res.write(`event: ${event}\n`)
      res.write(`data: ${payload}\n`)
      res.write(`id: ${Date.now()}\n\n`)
    } catch (error) {
      logger.error('SSE send to client failed', { error })
    }
  }

  getStats (): {totalUsers: number; totalConnections: number} {
    let totalConnections = 0
    this.clients.forEach((userClients) => {
      totalConnections += userClients.size
    })

    return {
      totalUsers: this.clients.size,
      totalConnections
    }
  }

  async cleanup (): Promise<void> {
    this.clients.forEach((userClients) => {
      userClients.forEach((res) => {
        if (res.writable) {res.end()}
      })
    })
    this.clients.clear()
    this.channelSubscribers.clear()

    await this.bridge.close()
    this.initialized = false

    logger.info('SSE service cleaned up')
  }
}

export default SSEService