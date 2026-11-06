import type { ConsumeMessage } from 'amqplib';
import type {
    EmailMessagePayload,
    SmsMessagePayload,
    DatabaseMessagePayload,
    PaymentMessagePayload,
    ImageMessagePayload
} from '../types/message.types';
import logger from '../utils/logger';

/**
 * Example: Email notification worker
 * Handler untuk mengirim email notifications
 */
export async function handleEmailMessage(content: EmailMessagePayload, message: ConsumeMessage): Promise<void> {
    logger.info('Processing email message...', {
        to: content.to,
        subject: content.subject,
        messageId: message.properties.messageId
    });

    // Simulate sending email
    await new Promise(resolve => setTimeout(resolve, 500));

    // Your email sending logic here
    // Example: await emailService.send(content);

    logger.info('Email sent successfully', { to: content.to });
}

/**
 * Example: SMS notification worker
 * Handler untuk mengirim SMS notifications
 */
export async function handleSmsMessage(content: SmsMessagePayload, message: ConsumeMessage): Promise<void> {
    logger.info('Processing SMS message...', {
        phone: content.phone,
        message: content.text,
        messageId: message.properties.messageId
    });

    // Simulate sending SMS
    await new Promise(resolve => setTimeout(resolve, 300));

    // Your SMS sending logic here
    // Example: await smsService.send(content);

    logger.info('SMS sent successfully', { phone: content.phone });
}

/**
 * Example: Database operation worker
 * Handler untuk operasi database (insert, update, delete)
 */
export async function handleDatabaseMessage(content: DatabaseMessagePayload, message: ConsumeMessage): Promise<void> {
    logger.info('Processing database operation...', {
        operation: content.operation,
        table: content.table,
        messageId: message.properties.messageId
    });

    // Simulate database operation
    await new Promise(resolve => setTimeout(resolve, 200));

    logger.info('Database operation completed successfully', {
        operation: content.operation,
        table: content.table
    });
}

/**
 * Example: Payment processing worker
 * Handler untuk memproses pembayaran
 */
export async function handlePaymentMessage(content: PaymentMessagePayload, message: ConsumeMessage): Promise<void> {
    logger.info('Processing payment...', {
        orderId: content.orderId,
        amount: content.amount,
        paymentMethod: content.paymentMethod,
        messageId: message.properties.messageId
    });

    try {
        // Validate payment
        if (!content.orderId || !content.amount) {
            throw new Error('Invalid payment data');
        }

        // Simulate payment processing
        await new Promise(resolve => setTimeout(resolve, 1000));

        logger.info('Payment processed successfully', { orderId: content.orderId });
    } catch (error) {
        logger.error('Payment processing failed', { error: String(error), orderId: content.orderId });
        throw error; // Will trigger requeue
    }
}

/**
 * Example: Image processing worker
 * Handler untuk memproses gambar (resize, compress, etc)
 */
export async function handleImageMessage(content: ImageMessagePayload, message: ConsumeMessage): Promise<void> {
    logger.info('Processing image...', {
        imageUrl: content.imageUrl,
        targetSizes: content.targetSizes,
        messageId: message.properties.messageId
    });

    // Simulate image processing
    await new Promise(resolve => setTimeout(resolve, 2000));

    logger.info('Image processed successfully', { imageUrl: content.imageUrl });
}

