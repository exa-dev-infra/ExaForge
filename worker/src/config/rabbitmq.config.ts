import dotenv from 'dotenv';

dotenv.config();

export interface RabbitMQConfig {
    url: string;
    queue: string;
    exchange: string;
    exchangeType: string;
    routingKey: string;
    prefetchCount: number;
    retryDelay: number;
}

export const rabbitmqConfig: RabbitMQConfig = {
    url: process.env.RABBITMQ_URL || 'amqp://localhost:5672',
    queue: process.env.RABBITMQ_QUEUE || 'my_queue',
    exchange: process.env.RABBITMQ_EXCHANGE || 'my_exchange',
    exchangeType: process.env.RABBITMQ_EXCHANGE_TYPE || 'direct',
    routingKey: process.env.RABBITMQ_ROUTING_KEY || 'my_routing_key',
    prefetchCount: parseInt(process.env.PREFETCH_COUNT || '10', 10),
    retryDelay: parseInt(process.env.RETRY_DELAY || '5000', 10),
};
