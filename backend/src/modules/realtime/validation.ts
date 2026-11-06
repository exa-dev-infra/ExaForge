import { z } from 'zod'

const eventBody = {
  event: z.string().min(1),
  data: z.any().optional()
}

export const notifyUserSchema = {
  params: z.object({
    userId: z.string().min(1)
  }),
  body: z.object(eventBody)
}

export const broadcastSchema = {
  body: z.object(eventBody)
}

export const channelSchema = {
  params: z.object({
    channel: z.string().min(1)
  }),
  body: z.object(eventBody)
}

export const onlineUserSchema = {
  params: z.object({
    userId: z.string().min(1)
  })
}

export const disconnectSchema = {
  params: z.object({
    userId: z.string().min(1)
  }),
  body: z.object({
    reason: z.string().optional()
  })
}