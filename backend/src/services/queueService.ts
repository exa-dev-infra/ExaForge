import RabbitMQClient from '../config/rabbitmq'
import logger from '../utils/logger'

export interface QueueMessage<T = any> {
    data: T;
    timestamp: Date;
    id?: string;
}

export class QueueService {
  private rabbitmq: RabbitMQClient

  constructor () {
    this.rabbitmq = RabbitMQClient.getInstance()
  }

  async sendEmail (to: string, subject: string, body: string): Promise<void> {
    const message: QueueMessage = {
      data: { to, subject, body },
      timestamp: new Date()
    }

    await this.rabbitmq.assertQueue('email_queue')
    await this.rabbitmq.sendToQueue('email_queue', message)
    logger.info(`Email queued for ${to}`)
  }

  async sendNotification (
    userId: string,
    type: 'info' | 'warning' | 'error' | 'success',
    message: string
  ): Promise<void> {
    const notification: QueueMessage = {
      data: { userId, type, message },
      timestamp: new Date()
    }

    await this.rabbitmq.assertQueue('notification_queue')
    await this.rabbitmq.sendToQueue('notification_queue', notification)
    logger.info(`Notification queued for user ${userId}`)
  }

  async sendToQueue<T> (queueName: string, data: T): Promise<void> {
    const message: QueueMessage<T> = {
      data,
      timestamp: new Date()
    }

    await this.rabbitmq.assertQueue(queueName)
    await this.rabbitmq.sendToQueue(queueName, message)
  }

  async publishEvent<T> (
    exchange: string,
    routingKey: string,
    data: T
  ): Promise<void> {
    const message: QueueMessage<T> = {
      data,
      timestamp: new Date()
    }

    await this.rabbitmq.assertExchange(exchange, 'topic')
    await this.rabbitmq.publish(exchange, routingKey, message)
  }
}

export default new QueueService()
