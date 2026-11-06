import amqp, { type Channel, type ConsumeMessage, type ChannelModel } from 'amqplib';
import type { RabbitMQConfig } from '../config/rabbitmq.config';
import type { MessageHandler } from '../types/message.types';
import logger from '../utils/logger';

export type { MessageHandler };

export class RabbitMQService {
    private connection: ChannelModel | null = null;
    private channel: Channel | null = null;
    private config: RabbitMQConfig;
    private isConnecting: boolean = false;

    constructor(config: RabbitMQConfig) {
        this.config = config;
    }

    /**
     * Connect to RabbitMQ and setup channel
     */
    async connect(): Promise<void> {
        if (this.isConnecting) {
            logger.warn('Connection already in progress...');
            return;
        }

        try {
            this.isConnecting = true;
            logger.info(`Connecting to RabbitMQ at ${this.config.url}...`);

            const connection = await amqp.connect(this.config.url);
            this.connection = connection;
            logger.info('Connected to RabbitMQ');

            const channel = await connection.createChannel();
            this.channel = channel;
            logger.info('Channel created');

            // Set prefetch count for fair dispatch
            await channel.prefetch(this.config.prefetchCount);

            // Setup exchange
            await channel.assertExchange(
                this.config.exchange,
                this.config.exchangeType,
                {durable: true}
            );
            logger.info(`Exchange asserted`, { exchange: this.config.exchange });

            // Setup queue
            await channel.assertQueue(this.config.queue, {durable: true});
            logger.info(`Queue asserted`, { queue: this.config.queue });

            // Bind queue to exchange
            await channel.bindQueue(
                this.config.queue,
                this.config.exchange,
                this.config.routingKey
            );
            logger.info(`Queue bound to exchange`, { routingKey: this.config.routingKey });

            // Handle connection errors
            connection.on('error', (err) => {
                logger.error('RabbitMQ connection error', { error: String(err) });
                this.reconnect();
            });

            connection.on('close', () => {
                logger.warn('RabbitMQ connection closed');
                this.reconnect();
            });

            this.isConnecting = false;
        } catch (error) {
            this.isConnecting = false;
            logger.error('Failed to connect to RabbitMQ', { error: String(error) });
            throw error;
        }
    }

    /**
     * Reconnect to RabbitMQ after delay
     */
    private async reconnect(): Promise<void> {
        if (this.isConnecting) {
            return;
        }

        logger.info(`Attempting to reconnect in ${this.config.retryDelay}ms...`);

        setTimeout(async () => {
            try {
                await this.connect();
            } catch (error) {
                logger.error('Reconnection failed', { error: String(error) });
                this.reconnect();
            }
        }, this.config.retryDelay);
    }

    /**
     * Start consuming messages from the queue
     */
    async consume<T = unknown>(handler: MessageHandler<T>): Promise<void> {
        if (!this.channel) {
            throw new Error('Channel not initialized. Call connect() first.');
        }

        logger.info(`Starting to consume messages from queue '${this.config.queue}'...`);

        await this.channel.consume(
            this.config.queue,
            async (message) => {
                if (!message) {
                    return;
                }

                try {
                    // Parse message content
                    const content = JSON.parse(message.content.toString()) as T;
                    logger.info('Received message', { content });

                    // Process message
                    await handler(content, message);

                    // Acknowledge message
                    this.channel?.ack(message);
                    logger.info('Message acknowledged');
                } catch (error) {
                    logger.error('Error processing message', { error: String(error) });

                    // Reject and requeue the message
                    this.channel?.nack(message, false, true);
                    logger.warn('Message rejected and requeued');
                }
            },
            {noAck: false}
        );

        logger.info('Consumer started successfully');
    }

    /**
     * Close connection and channel
     */
    async close(): Promise<void> {
        try {
            if (this.channel) {
                await this.channel.close();
                logger.info('Channel closed');
            }

            if (this.connection) {
                await this.connection.close();
                logger.info('Connection closed');
            }
        } catch (error) {
            logger.error('Error closing RabbitMQ connection', { error: String(error) });
        }
    }

    /**
     * Check if connected
     */
    isConnected(): boolean {
        return this.connection !== null && this.channel !== null;
    }
}
