import { z } from 'zod'
import { file } from '../../utils/validator'

export const uploadDocumentSchema = {
  body: z.object({
    title: z.string().min(2).max(100),
    description: z.string().max(500).optional(),
    tags: z.array(z.string().min(1)).min(1).max(10).optional()
  }),
  files: z.object({
    avatar: file({ allowedMime: ['image/png', 'image/jpeg'], maxBytes: 1024 * 1024 }).optional(),
    documents: file({ maxBytes: 5 * 1024 * 1024, multiple: true }).optional()
  })
}