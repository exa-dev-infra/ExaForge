import { z } from 'zod'
import { numeric } from '../../utils/validator'

export const redisExampleSchema = {
  body: z.object({
    key: z.string().min(1),
    value: z.string().min(1),
    expiry: numeric().optional()
  })
}

export const rabbitmqSendSchema = {
  body: z.object({
    queue: z.string().min(1),
    message: z.string().min(1)
  })
}