import { z } from 'zod'
import { numeric } from '../../utils/validator'

export const listUsersSchema = {
  query: z.object({
    page: numeric(1).optional(),
    limit: numeric(1, 100).optional()
  })
}

export const userIdSchema = {
  params: z.object({
    id: z.string().min(1)
  })
}

export const createUserSchema = {
  body: z.object({
    name: z.string().min(2).max(100),
    email: z.string().email(),
    password: z.string().min(8).max(72).optional(),
    role: z.string().optional()
  })
}

export const updateUserSchema = {
  params: z.object({
    id: z.string().min(1)
  }),
  body: z.object({
    name: z.string().min(2).max(100).optional(),
    email: z.string().email().optional(),
    password: z.string().min(8).max(72).optional(),
    role: z.string().optional()
  })
}