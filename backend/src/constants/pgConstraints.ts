// PostgreSQL constraint name to field name mapping
// This helps to convert database constraint errors to user-friendly field names
export const UNIQUE_CONSTRAINT_FIELD_MAP: Record<string, string> = {
  users_email_key: 'email',
  refresh_tokens_token_hash_key: 'token'
}

// Foreign key constraint mapping
export const FOREIGN_KEY_CONSTRAINT_MAP: Record<string, string> = {
  refresh_tokens_user_id_fkey: 'user_id'
}

// Check constraint mapping
export const CHECK_CONSTRAINT_MAP: Record<string, string> = {
  users_age_check: 'age'
}

// PostgreSQL error codes
export const PG_ERROR_CODES = {
  UNIQUE_VIOLATION: '23505',
  FOREIGN_KEY_VIOLATION: '23503',
  NOT_NULL_VIOLATION: '23502',
  CHECK_VIOLATION: '23514',
  INVALID_TEXT_REPRESENTATION: '22P02',
  NUMERIC_VALUE_OUT_OF_RANGE: '22003',
  DIVISION_BY_ZERO: '22012',
  UNDEFINED_TABLE: '42P01',
  UNDEFINED_COLUMN: '42703',
  SYNTAX_ERROR: '42601',
  DUPLICATE_TABLE: '42P07',
  DUPLICATE_COLUMN: '42701'
}
