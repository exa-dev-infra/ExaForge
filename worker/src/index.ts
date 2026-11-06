import {RabbitMQService} from './services/rabbitmq.service';
import {rabbitmqConfig} from './config/rabbitmq.config';
import {handleMessage} from './workers/message.worker';
import logger from './utils/logger';

async function main() {
    logger.info('Starting RabbitMQ Worker...');
    logger.info('Configuration loaded', {
        url: rabbitmqConfig.url,
        queue: rabbitmqConfig.queue,
        exchange: rabbitmqConfig.exchange,
        routingKey: rabbitmqConfig.routingKey,
    });

    const rabbitMQService = new RabbitMQService(rabbitmqConfig);

    try {
        // Connect to RabbitMQ
        await rabbitMQService.connect();

        // Start consuming messages
        await rabbitMQService.consume(handleMessage);

        logger.info('Worker is running and waiting for messages...');
    } catch (error) {
        logger.error('Failed to start worker', { error: String(error) });
        process.exit(1);
    }

    // Graceful shutdown
    const shutdown = async () => {
        logger.info('Shutting down worker...');
        await rabbitMQService.close();
        logger.info('Worker stopped');
        process.exit(0);
    };

    process.on('SIGINT', shutdown);
    process.on('SIGTERM', shutdown);
}

main();
