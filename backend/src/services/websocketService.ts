import type { Server as HTTPServer } from 'http'
import type { Socket } from 'socket.io'
import { Server as SocketIOServer } from 'socket.io'
import type { RealtimeBridge } from './realtime'
import { createBridge } from './realtime'
import jwtService from './jwtService'
import logger from '../utils/logger'

interface SocketUser {
  userId: string
  email?: string
}

/**
 * WebSocket Service - horizontally scalable through the same realtime bridge
 * used by SSE. Every fan-out goes through the bridge (Redis or RabbitMQ): the
 * node that emits publishes the message and, on the receiving side, every node
 * forwards it only to its own locally-connected sockets.
 */
class WebSocketService {
  private static instance: WebSocketService
  private io: SocketIOServer | null = null
  private bridge: RealtimeBridge
  private connectedUsers: Map<string, Set<string>> = new Map()
  private initialized = false

  private constructor () {
    this.bridge = createBridge()
  }

  public static getInstance (): WebSocketService {
    if (!WebSocketService.instance) {
      WebSocketService.instance = new WebSocketService()
    }
    return WebSocketService.instance
  }

  async initialize (httpServer: HTTPServer): Promise<void> {
    this.io = new SocketIOServer(httpServer, {
      cors: {
        origin: process.env.CORS_ORIGIN || '*',
        methods: ['GET', 'POST'],
        credentials: true
      },
      transports: ['websocket', 'polling'],
      pingTimeout: 60000,
      pingInterval: 25000
    })

    this.io.use(this.authMiddleware.bind(this))
    this.io.on('connection', this.handleConnection.bind(this))

    await this.bridge.init()
    await this.bridge.subscribe('ws:*', this.handleBridgeMessage.bind(this))

    this.initialized = true
    logger.info('WebSocket service initialized with realtime bridge')
  }

  private authMiddleware (socket: Socket, next: (err?: Error) => void): void {
    try {
      const handshakeToken = socket.handshake.auth?.token
      const authorization = (socket.handshake.headers as Record<string, unknown>).authorization

      const token = typeof handshakeToken === 'string'
        ? handshakeToken
        : typeof authorization === 'string'
          ? authorization.split(' ')[1]
          : undefined

      if (!token) {
        next(new Error('Authentication token required'))
        return
      }

      const decoded = jwtService.verifyAccessToken(token)

      ;(socket.data as SocketUser).userId = String(decoded.userId)
      ;(socket.data as SocketUser).email = decoded.email

      next()
    } catch {
      next(new Error('Authentication failed'))
    }
  }

  private handleConnection (socket: Socket): void {
    const { userId } = socket.data as SocketUser

    logger.debug('WebSocket client connected', { userId, socketId: socket.id })

    if (!this.connectedUsers.has(userId)) {
      this.connectedUsers.set(userId, new Set())
    }
    this.connectedUsers.get(userId)!.add(socket.id)

    socket.join(`user:${userId}`)

    socket.emit('connected', {
      message: 'Connected to WebSocket',
      userId,
      socketId: socket.id,
      timestamp: new Date().toISOString()
    })

    this.setupEventHandlers(socket)

    socket.on('disconnect', () => {
      this.handleDisconnect(socket)
    })
  }

  private setupEventHandlers (socket: Socket): void {
    const { userId } = socket.data as SocketUser

    socket.on('join:room', (roomId: string) => {
      socket.join(`room:${roomId}`)
      socket.emit('joined:room', { roomId, timestamp: new Date().toISOString() })
    })

    socket.on('leave:room', (roomId: string) => {
      socket.leave(`room:${roomId}`)
      socket.emit('left:room', { roomId, timestamp: new Date().toISOString() })
    })

    socket.on('subscribe', (channel: string) => {
      socket.join(`channel:${channel}`)
      socket.emit('subscribed', { channel, timestamp: new Date().toISOString() })
    })

    socket.on('unsubscribe', (channel: string) => {
      socket.leave(`channel:${channel}`)
      socket.emit('unsubscribed', { channel, timestamp: new Date().toISOString() })
    })

    socket.on('message', (data: unknown) => {
      socket.emit('message:received', {
        success: true,
        timestamp: new Date().toISOString()
      })
      logger.debug('WebSocket message received', { userId, data })
    })

    socket.on('ping', () => {
      socket.emit('pong', { timestamp: new Date().toISOString() })
    })
  }

  private handleDisconnect (socket: Socket): void {
    const { userId } = socket.data as SocketUser

    logger.info('WebSocket disconnected', { userId, socketId: socket.id })

    const userSockets = this.connectedUsers.get(userId)
    if (userSockets) {
      userSockets.delete(socket.id)
      if (userSockets.size === 0) {
        this.connectedUsers.delete(userId)
      }
    }
  }

  sendToUser (userId: string | number, event: string, data: unknown): Promise<void> {
    return this.publish(`ws:user:${String(userId)}`, event, data)
  }

  sendToRoom (roomId: string | number, event: string, data: unknown): Promise<void> {
    return this.publish(`ws:room:${String(roomId)}`, event, data)
  }

  sendToChannel (channel: string, event: string, data: unknown): Promise<void> {
    return this.publish(`ws:channel:${channel}`, event, data)
  }

  broadcast (event: string, data: unknown): Promise<void> {
    return this.publish('ws:broadcast', event, data)
  }

  private async publish (channel: string, event: string, data: unknown): Promise<void> {
    const payload = JSON.stringify({ event, data })
    if (this.initialized) {
      await this.bridge.publish(channel, payload)
      return
    }
    // Bridge not ready: emit only to local sockets.
    this.emitLocal(channel, event, data)
  }

  private handleBridgeMessage = (channelName: string, message: string): void => {
    try {
      const parsed = JSON.parse(message) as {event: string; data: unknown}
      this.emitLocal(channelName, parsed.event, parsed.data)
    } catch (error) {
      logger.error('WebSocket bridge message handling failed', { error })
    }
  }

  private emitLocal (channel: string, event: string, data: unknown): void {
    if (!this.io) {return}

    if (channel === 'ws:broadcast') {
      this.io.emit(event, data)
      return
    }

    const prefixes: Array<[string, string]> = [
      ['ws:user:', 'user:'],
      ['ws:room:', 'room:'],
      ['ws:channel:', 'channel:']
    ]

    for (const [prefix, roomPrefix] of prefixes) {
      if (channel.startsWith(prefix)) {
        const id = channel.slice(prefix.length)
        this.io.to(`${roomPrefix}${id}`).emit(event, data)
        return
      }
    }
  }

  async getStats (): Promise<{
    totalUsers: number
    totalConnections: number
    onlineUsers: string[]
  }> {
    const sockets = this.io ? await this.io.fetchSockets() : []

    return {
      totalUsers: this.connectedUsers.size,
      totalConnections: sockets.length,
      onlineUsers: Array.from(this.connectedUsers.keys())
    }
  }

  isUserOnline (userId: string | number): boolean {
    return this.connectedUsers.has(String(userId))
  }

  async disconnectUser (userId: string | number, reason?: string): Promise<void> {
    if (!this.io) {return}

    const sockets = await this.io.in(`user:${String(userId)}`).fetchSockets()
    sockets.forEach((socket) => socket.disconnect(true))

    logger.info('WebSocket user disconnected', { userId, reason: reason || 'Manual disconnect' })
  }

  getIO (): SocketIOServer | null {
    return this.io
  }

  async cleanup (): Promise<void> {
    if (this.io) {
      this.io.disconnectSockets()
      this.io.close()
      this.io = null
    }

    this.connectedUsers.clear()
    await this.bridge.close()
    this.initialized = false

    logger.info('WebSocket service cleaned up')
  }
}

export default WebSocketService