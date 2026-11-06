import { z } from 'zod'
import { numeric } from '../../utils/validator'

export const registerSchema = {
  body: z.object({
    name: z.string().min(2).max(100),
    email: z.string().email(),
    password: z.string().min(8).max(72),
    role: z.string().optional()
  })
}

export const loginSchema = {
  body: z.object({
    email: z.string().email(),
    password: z.string().min(1)
  })
}

export const refreshSchema = {
  body: z.object({
    refreshToken: z.string().optional()
  }).optional()
}

export const logoutSchema = {
  body: z.object({
    userId: numeric().optional(),
    refreshToken: z.string().optional()
  }).optional()
}

export const changePasswordSchema = {
  body: z.object({
    currentPassword: z.string().min(1),
    newPassword: z.string().min(8).max(72)
  })
}