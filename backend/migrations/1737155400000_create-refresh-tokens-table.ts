/* eslint-disable camelcase */
import type { ColumnDefinitions, MigrationBuilder } from 'node-pg-migrate'

export const shorthands: ColumnDefinitions | undefined = undefined

export async function up (pgm: MigrationBuilder): Promise<void> {
  pgm.createTable('refresh_tokens', {
    id: {
      type: 'serial',
      primaryKey: true
    },
    user_id: {
      type: 'integer',
      notNull: true,
      references: 'users(id)',
      onDelete: 'CASCADE'
    },
    token_hash: {
      type: 'varchar(255)',
      notNull: true,
      unique: true
    },
    expires_at: {
      type: 'timestamptz',
      notNull: true
    },
    revoked_at: {
      type: 'timestamptz',
      notNull: false
    },
    user_agent: {
      type: 'varchar(500)',
      notNull: false
    },
    ip_address: {
      type: 'varchar(45)',
      notNull: false
    },
    created_at: {
      type: 'timestamptz',
      notNull: true,
      default: pgm.func('current_timestamp')
    },
    updated_at: {
      type: 'timestamptz',
      notNull: true,
      default: pgm.func('current_timestamp')
    }
  })

  pgm.createIndex('refresh_tokens', 'user_id')
  pgm.createIndex('refresh_tokens', 'token_hash')
}

export async function down (pgm: MigrationBuilder): Promise<void> {
  pgm.dropTable('refresh_tokens')
}
