import type { Channel, Connection, ConsumeMessage } from 'amqplib'
import amqp from 'amqplib'
import dotenv from 'dotenv'
import logger from '../utils/logger'

dotenv.config()

interface QueueOptions {
    durable?: boolean;
    autoDelete?: boolean;
    exclusive?: boolean;
}

interface ExchangeOptions {
    durable?: boolean;
    autoDelete?: boolean;
    type?: 'direct' | 'topic' | 'fanout' | 'headers';
}

class RabbitMQClient {
  private connection: Connection | null = null
  private channel: Channel | null = null
  private static instance: RabbitMQClient
  private isConnected: boolean = false
  private reconnectTimeout: NodeJS.Timeout | null = null
  private intentionalDisconnect: boolean = false

  private constructor () {
  }

  public static getInstance (): RabbitMQClient {
    if (!RabbitMQClient.instance) {
      RabbitMQClient.instance = new RabbitMQClient()
    }
    return RabbitMQClient.instance
  }

  public async connect (): Promise<void> {
    try {
      const rabbitUrl = process.env.RABBITMQ_URL || 'amqp://guest:guest@localhost:5672'

      this.connection = await amqp.connect(rabbitUrl) as unknown as Connection
      logger.info('RabbitMQ connection established')

      if (this.connection) {
        this.connection.on('error', (err) => {
          logger.error('RabbitMQ connection error:', { err })
          this.isConnected = false
          if (!this.intentionalDisconnect) {
            this.reconnect()
          }
        })

        this.connection.on('close', () => {
          logger.info('RabbitMQ connection closed')
          this.isConnected = false
          if (!this.intentionalDisconnect) {
            this.reconnect()
          }
        })

        this.channel = await (this.connection as any).createChannel()
        logger.info('RabbitMQ channel created')

        if (this.channel) {
          this.channel.on('error', (err) => {
            logger.error('RabbitMQ channel error:', { err })
          })

          this.channel.on('close', () => {
            logger.info('RabbitMQ channel closed')
          })
        }
      }

      this.isConnected = true
    } catch (error) {
      logger.error('Failed to connect to RabbitMQ:', { error })
      this.reconnect()
      throw error
    }
  }

  private reconnect (): void {
    if (this.reconnectTimeout) {
      return
    }

    this.reconnectTimeout = setTimeout(async () => {
      logger.info('Attempting to reconnect to RabbitMQ...')
      this.reconnectTimeout = null
      try {
        await this.connect()
      } catch (error) {
        logger.error('Reconnection failed:', { error })
      }
    }, 5000)
  }

  public async disconnect (): Promise<void> {
    this.intentionalDisconnect = true
    try {
      // Clear any pending reconnection attempt
      if (this.reconnectTimeout) {
        clearTimeout(this.reconnectTimeout)
        this.reconnectTimeout = null
      }
      if (this.channel) {
        await this.channel.close()
        this.channel = null
      }
      if (this.connection) {
        await (this.connection as any).close()
        this.connection = null
      }
      this.isConnected = false
      logger.info('RabbitMQ disconnected')
    } catch (error) {
      logger.error('Error disconnecting from RabbitMQ:', { error })
    }
  }

  public getChannel (): Channel {
    if (!this.channel || !this.isConnected) {
      throw new Error('RabbitMQ channel is not available')
    }
    return this.channel
  }

  // Queue operations
  public async assertQueue (queue: string, options: QueueOptions = { durable: true }): Promise<void> {
    const channel = this.getChannel()
    await channel.assertQueue(queue, options)
    logger.info(`Queue "${queue}" asserted`)
  }

  public async sendToQueue (queue: string, message: any): Promise<boolean> {
    const channel = this.getChannel()
    const messageBuffer = Buffer.from(JSON.stringify(message))
    return channel.sendToQueue(queue, messageBuffer, { persistent: true })
  }

  public async consume (
    queue: string,
    callback: (msg: ConsumeMessage | null) => void,
    options: { noAck?: boolean } = { noAck: false }
  ): Promise<void> {
    const channel = this.getChannel()
    await channel.consume(queue, callback, options)
    logger.info(`Consuming messages from queue "${queue}"`)
  }

  public ack (message: ConsumeMessage): void {
    const channel = this.getChannel()
    channel.ack(message)
  }

  public nack (message: ConsumeMessage, allUpTo: boolean = false, requeue: boolean = true): void {
    const channel = this.getChannel()
    channel.nack(message, allUpTo, requeue)
  }

  // Exchange operations
  public async assertExchange (
    exchange: string,
    type: 'direct' | 'topic' | 'fanout' | 'headers' = 'direct',
    options: ExchangeOptions = { durable: true }
  ): Promise<void> {
    const channel = this.getChannel()
    await channel.assertExchange(exchange, type, options)
    logger.info(`Exchange "${exchange}" asserted with type "${type}"`)
  }

  public async bindQueue (queue: string, exchange: string, routingKey: string = ''): Promise<void> {
    const channel = this.getChannel()
    await channel.bindQueue(queue, exchange, routingKey)
    logger.info(`Queue "${queue}" bound to exchange "${exchange}" with routing key "${routingKey}"`)
  }

  public async publish (exchange: string, routingKey: string, message: any): Promise<boolean> {
    const channel = this.getChannel()
    const messageBuffer = Buffer.from(JSON.stringify(message))
    return channel.publish(exchange, routingKey, messageBuffer, { persistent: true })
  }

  // Prefetch settings
  public async setPrefetch (count: number): Promise<void> {
    const channel = this.getChannel()
    await channel.prefetch(count)
    logger.info(`Prefetch count set to ${count}`)
  }
}

export default RabbitMQClient
