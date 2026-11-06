import { z } from 'zod'

export const sseTestSchema = {
  body: z.object({
    name: z.string().optional(),
    message: z.string().optional(),
    data: z.any().optional()
  })
}