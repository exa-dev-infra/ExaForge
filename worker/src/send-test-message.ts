import amqp from 'amqplib';
import dotenv from 'dotenv';
import logger from './utils/logger';

dotenv.config();

/**
 * Script untuk mengirim test message ke RabbitMQ
 * Gunakan ini untuk testing worker
 */
async function sendTestMessage() {
    try {
        const url = process.env.RABBITMQ_URL || 'amqp://localhost:5672';
        const exchange = process.env.RABBITMQ_EXCHANGE || 'my_exchange';
        const routingKey = process.env.RABBITMQ_ROUTING_KEY || 'my_routing_key';

        logger.info('Connecting to RabbitMQ...', { url });
        const connection = await amqp.connect(url);
        const channel = await connection.createChannel();

        // Assert exchange
        await channel.assertExchange(exchange, 'direct', {durable: true});

        // Sample messages
        const messages = [
            {
                id: 1,
                name: 'Test Message 1',
                data: {user: 'John Doe', action: 'login'},
                timestamp: new Date().toISOString(),
            },
            {
                id: 2,
                name: 'Test Message 2',
                data: {user: 'Jane Smith', action: 'purchase', amount: 150},
                timestamp: new Date().toISOString(),
            },
            {
                id: 3,
                name: 'Test Message 3',
                data: {user: 'Bob Johnson', action: 'logout'},
                timestamp: new Date().toISOString(),
            },
        ];

        // Send each message
        for (const message of messages) {
            channel.publish(
                exchange,
                routingKey,
                Buffer.from(JSON.stringify(message)),
                {
                    persistent: true,
                    messageId: message.id.toString(),
                }
            );

            logger.info('Message sent', { message });
        }

        logger.info('Total messages sent successfully', { count: messages.length });

        // Close connection
        await channel.close();
        await connection.close();
    } catch (error) {
        logger.error('Error sending message', { error: String(error) });
        process.exit(1);
    }
}

sendTestMessage();
