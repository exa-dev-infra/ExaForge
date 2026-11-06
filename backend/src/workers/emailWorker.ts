import RabbitMQClient from '../config/rabbitmq'
import RedisClient from '../config/redis'
import Database from '../config/database'
import type { ConsumeMessage } from 'amqplib'
import logger from '../utils/logger'

interface EmailMessage {
    to: string;
    subject: string;
    body: string;
}

class EmailWorker {
  private rabbitmq: RabbitMQClient
  private redis: RedisClient
  private db: Database
  private queueName = 'email_queue'

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

      // Set prefetch to 1 to process one message at a time
      await this.rabbitmq.setPrefetch(1)

      // Start consuming messages
      await this.rabbitmq.consume(this.queueName, this.processMessage.bind(this))

      logger.info(`📧 Email Worker started, listening to queue: ${this.queueName}`)
    } catch (error) {
      logger.error('Failed to start Email Worker:', { error })
      throw error
    }
  }

  private async processMessage (msg: ConsumeMessage | null): Promise<void> {
    if (!msg) {
      return
    }

    try {
      const content = msg.content.toString()
      const emailData: EmailMessage = JSON.parse(content)

      logger.info('Processing email:', { emailData })

      // Simulate email sending
      await this.sendEmail(emailData)

      // Store in Redis for tracking
      const cacheKey = `email:sent:${emailData.to}:${Date.now()}`
      await this.redis.set(cacheKey, JSON.stringify(emailData), 3600) // Cache for 1 hour

      // Log to database (optional)
      await this.db.withTransaction(async (client) => {
        await client.query(
          'INSERT INTO email_logs (recipient, subject, status) VALUES ($1, $2, $3)',
          [emailData.to, emailData.subject, 'sent']
        )
      })

      // Acknowledge the message
      this.rabbitmq.ack(msg)

      logger.info('✓ Email processed successfully:', { to: emailData.to })
    } catch (error) {
      logger.error('Error processing message:', { error })

      // Reject and requeue the message
      this.rabbitmq.nack(msg, false, true)
    }
  }

  private async sendEmail (emailData: EmailMessage): Promise<void> {
    // Simulate email sending delay
    await new Promise(resolve => setTimeout(resolve, 1000))

    // Here you would integrate with actual email service (SendGrid, AWS SES, etc.)
    logger.info(`Sending email to ${emailData.to}...`, {
      subject: emailData.subject,
      body: emailData.body
    })
  }

  async stop (): Promise<void> {
    await this.rabbitmq.disconnect()
    await this.redis.disconnect()
    await this.db.close()
    logger.info('Email Worker stopped')
  }
}

// Start worker if run directly
if (require.main === module) {
  const worker = new EmailWorker()

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

export default EmailWorker
