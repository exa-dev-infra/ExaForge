import RabbitMQClient from '../config/rabbitmq'
import RedisClient from '../config/redis'
import Database from '../config/database'
import type { ConsumeMessage } from 'amqplib'
import logger from '../utils/logger'

interface NotificationMessage {
    userId: string;
    type: 'info' | 'warning' | 'error' | 'success';
    message: string;
    metadata?: Record<string, any>;
}

class NotificationWorker {
  private rabbitmq: RabbitMQClient
  private redis: RedisClient
  private db: Database
  private queueName = 'notification_queue'

  constructor () {
    this.rabbitmq = RabbitMQClient.getInstance()
    this.redis = RedisClient.getInstance()
    this.db = Database.getInstance()
  }

  async start (): Promise<void> {
    try {
      // Connect to services
      await this.rabbitmq.connect()
      await this.redis.connect()

      // Assert queue
      await this.rabbitmq.assertQueue(this.queueName)

      // Set prefetch
      await this.rabbitmq.setPrefetch(5)

      // Start consuming messages
      await this.rabbitmq.consume(this.queueName, this.processMessage.bind(this))

      logger.info(`🔔 Notification Worker started, listening to queue: ${this.queueName}`)
    } catch (error) {
      logger.error('Failed to start Notification Worker:', { error })
      throw error
    }
  }

  private async processMessage (msg: ConsumeMessage | null): Promise<void> {
    if (!msg) {
      return
    }

    try {
      const content = msg.content.toString()
      const notificationData: NotificationMessage = JSON.parse(content)

      logger.info('Processing notification:', { notificationData })

      // Process notification
      await this.sendNotification(notificationData)

      // Cache notification in Redis
      const cacheKey = `notification:${notificationData.userId}:${Date.now()}`
      await this.redis.hSet(cacheKey, 'type', notificationData.type)
      await this.redis.hSet(cacheKey, 'message', notificationData.message)
      await this.redis.expire(cacheKey, 86400) // 24 hours

      // Acknowledge the message
      this.rabbitmq.ack(msg)

      logger.info('✓ Notification processed successfully')
    } catch (error) {
      logger.error('Error processing notification:', { error })
      this.rabbitmq.nack(msg, false, true)
    }
  }

  private async sendNotification (notificationData: NotificationMessage): Promise<void> {
    // Simulate notification sending
    await new Promise(resolve => setTimeout(resolve, 500))

    logger.info(`📱 Sending ${notificationData.type} notification to user ${notificationData.userId}`, {
      message: notificationData.message
    })

    // Here you would integrate with push notification service (FCM, APNS, etc.)
  }

  async stop (): Promise<void> {
    await this.rabbitmq.disconnect()
    await this.redis.disconnect()
    await this.db.close()
    logger.info('Notification Worker stopped')
  }
}

// Start worker if run directly
if (require.main === module) {
  const worker = new NotificationWorker()

  worker.start().catch(error => {
    logger.error('Worker failed:', { error })
    process.exit(1)
  })

  // Graceful shutdown
  const shutdown = async () => {
    logger.info('\nShutting down worker...')
    await worker.stop()
    process.exit(0)
  }

  process.on('SIGTERM', shutdown)
  process.on('SIGINT', shutdown)
}

export default NotificationWorker
