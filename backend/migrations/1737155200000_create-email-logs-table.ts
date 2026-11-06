/* eslint-disable camelcase */
import {ColumnDefinitions, MigrationBuilder} from 'node-pg-migrate';

export const shorthands: ColumnDefinitions | undefined = undefined;

export async function up(pgm: MigrationBuilder): Promise<void> {
    // Create email_logs table
    pgm.createTable('email_logs', {
        id: {
            type: 'serial',
            primaryKey: true,
        },
        recipient: {
            type: 'varchar(255)',
            notNull: true,
        },
        subject: {
            type: 'varchar(500)',
            notNull: true,
        },
        status: {
            type: 'varchar(50)',
            notNull: true,
        },
        error_message: {
            type: 'text',
        },
        created_at: {
            type: 'timestamp',
            notNull: true,
            default: pgm.func('current_timestamp'),
        },
    });

    // Create index on recipient and status
    pgm.createIndex('email_logs', 'recipient');
    pgm.createIndex('email_logs', 'status');
    pgm.createIndex('email_logs', 'created_at');
}

export async function down(pgm: MigrationBuilder): Promise<void> {
    pgm.dropTable('email_logs');
}
