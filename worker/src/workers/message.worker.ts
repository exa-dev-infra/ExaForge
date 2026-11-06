import type { ConsumeMessage } from 'amqplib';
import type { DefaultMessagePayload } from '../types/message.types';
import logger from '../utils/logger';

/**
 * Message handler - process your business logic here
 */
export async function handleMessage(content: DefaultMessagePayload, message: ConsumeMessage): Promise<void> {
    logger.info('Processing message...', {
        messageId: message.properties.messageId,
        content
    });

    // Simulate processing time
    await new Promise(resolve => setTimeout(resolve, 1000));

    // Your business logic here
    // Example: Save to database, call external API, transform data, etc.

    logger.info('Message processed successfully', {
        messageId: message.properties.messageId
    });
}
