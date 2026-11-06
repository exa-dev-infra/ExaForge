export interface LogContext {
  [key: string]: unknown;
}

class Logger {
  private format(level: string, message: string, context?: LogContext): string {
    return JSON.stringify({
      timestamp: new Date().toISOString(),
      level,
      message,
      ...(context || {})
    });
  }

  debug(message: string, context?: LogContext): void {
    process.stdout.write(this.format('DEBUG', message, context) + '\n');
  }

  info(message: string, context?: LogContext): void {
    process.stdout.write(this.format('INFO', message, context) + '\n');
  }

  warn(message: string, context?: LogContext): void {
    process.stderr.write(this.format('WARN', message, context) + '\n');
  }

  error(message: string, context?: LogContext): void {
    process.stderr.write(this.format('ERROR', message, context) + '\n');
  }
}

export const logger = new Logger();
export default logger;

