/* eslint-disable camelcase */
import {ColumnDefinitions, MigrationBuilder} from 'node-pg-migrate';

export const shorthands: ColumnDefinitions | undefined = undefined;

export async function up(pgm: MigrationBuilder): Promise<void> {
    // Add password column to users table
    pgm.addColumn('users', {
        password: {
            type: 'varchar(255)',
            notNull: false, // Allow null for existing users
        },
    });

    // Add role column to users table
    pgm.addColumn('users', {
        role: {
            type: 'varchar(50)',
            notNull: true,
            default: 'user',
        },
    });

    // Add last_login column to users table
    pgm.addColumn('users', {
        last_login: {
            type: 'timestamp',
            notNull: false,
        },
    });

    // Create index on role
    pgm.createIndex('users', 'role');
}

export async function down(pgm: MigrationBuilder): Promise<void> {
    pgm.dropColumn('users', 'password');
    pgm.dropColumn('users', 'role');
    pgm.dropColumn('users', 'last_login');
}
