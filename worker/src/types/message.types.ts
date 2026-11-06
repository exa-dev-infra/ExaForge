import type { ConsumeMessage } from 'amqplib';

export interface BaseMessagePayload {
    timestamp?: string | number;
    correlationId?: string;
}

export interface DefaultMessagePayload extends BaseMessagePayload {
    [key: string]: unknown;
}

export interface EmailMessagePayload extends BaseMessagePayload {
    to: string;
    subject: string;
    body: string;
}

export interface SmsMessagePayload extends BaseMessagePayload {
    phone: string;
    text: string;
}

export interface DatabaseMessagePayload extends BaseMessagePayload {
    operation: 'insert' | 'update' | 'delete';
    table: string;
    data: Record<string, unknown>;
}

export interface PaymentMessagePayload extends BaseMessagePayload {
    orderId: string;
    amount: number;
    currency: string;
    paymentMethod: string;
    customerEmail: string;
}

export interface ImageMessagePayload extends BaseMessagePayload {
    imageUrl: string;
    targetSizes: string[];
    format?: 'jpeg' | 'png' | 'webp';
}

export type MessageHandler<T = DefaultMessagePayload> = (content: T, message: ConsumeMessage) => Promise<void>;
