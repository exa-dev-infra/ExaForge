import RabbitMQClient from '../../config/rabbitmq'
import { features } from '../../config'
import type { RealtimeBridge, RealtimeHandler } from './bridge'
import { patternToRegExp } from './bridge'

const EXCHANGE = 'realtime'

interface Subscription {
  pattern: string
  regexp: RegExp
  handler: RealtimeHandler
}

/**
 * RabbitMQ topic-exchange implementation of RealtimeBridge.
 *
 * Publishers send to the `realtime` topic exchange using the channel name as
 * the routing key. Every process binds an exclusive queue to the exchange with
 * the `#` wildcard and dispatches matching messages to its local handlers.
 */
export class RabbitMQBridge implements RealtimeBridge {
  private rabbitmq: RabbitMQClient
  private subscriptions: Subscription[] = []
  private consumerTag: string | null = null
  private queue: string | null = null
  private channel: any = null

  constructor () {
    this.rabbitmq = RabbitMQClient.getInstance()
  }

  async init (): Promise<void> {
    if (!features.rabbitmq) {
      throw new Error('RabbitMQ disabled but REALTIME_BRIDGE is set to "rabbitmq"')
    }

    this.channel = this.rabbitmq.getChannel()
    await this.rabbitmq.assertExchange(EXCHANGE, 'topic', { durable: false })
  }

  async publish (channel: string, payload: string): Promise<void> {
    this.channel.publish(EXCHANGE, channel, Buffer.from(payload), { persistent: false })
  }

  async subscribe (pattern: string, handler: RealtimeHandler): Promise<void> {
    const subscription: Subscription = {
      pattern,
      regexp: patternToRegExp(pattern),
      handler
    }
    this.subscriptions.push(subscription)

    if (!this.queue) {
      await this.consumeAll()
    }
  }

  private async consumeAll (): Promise<void> {
    const declared = await this.channel.assertQueue('', { exclusive: true })
    this.queue = declared.queue

    await this.channel.bindQueue(this.queue, EXCHANGE, '#')

    const { consumerTag } = await this.channel.consume(
      this.queue,
      (msg: any) => {
        if (!msg) {return}
        const channelName = msg.fields.routingKey as string
        const payload = msg.content.toString()

        for (const subscription of this.subscriptions) {
          if (subscription.regexp.test(channelName)) {
            subscription.handler(channelName, payload)
          }
        }
      },
      { noAck: true }
    )

    this.consumerTag = consumerTag
  }

  async close (): Promise<void> {
    const channel = this.channel

    if (!channel || !this.queue) {return}

    try {
      if (this.consumerTag) {
        await channel.cancel(this.consumerTag)
      }
      await channel.deleteQueue(this.queue)
    } catch {
      // best effort cleanup
    } finally {
      this.consumerTag = null
      this.queue = null
    }
  }
}