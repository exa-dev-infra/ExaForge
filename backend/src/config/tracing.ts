import { NodeSDK } from '@opentelemetry/sdk-node'
import { getNodeAutoInstrumentations } from '@opentelemetry/auto-instrumentations-node'
import { OTLPTraceExporter } from '@opentelemetry/exporter-trace-otlp-grpc'
import dotenv from 'dotenv'
import logger from '../utils/logger'

dotenv.config()

let sdk: NodeSDK | null = null

export const initializeTracing = () => {
  if (process.env.OTEL_ENABLED !== 'true') {
    return
  }

  const traceExporter = new OTLPTraceExporter({
    url: process.env.OTEL_EXPORTER_OTLP_ENDPOINT || 'http://localhost:4317'
  })

  sdk = new NodeSDK({
    traceExporter,
    instrumentations: [getNodeAutoInstrumentations()]
  })

  sdk.start()
  logger.info('OpenTelemetry tracing initialized')

  process.on('SIGTERM', () => {
    sdk?.shutdown()
      .then(() => logger.info('Tracing terminated'))
      .catch((error) => logger.error('Error terminating tracing', { error }))
  })
}
