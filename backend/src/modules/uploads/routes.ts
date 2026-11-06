import type { Request, Response } from 'express'
import { Router } from 'express'
import { ApiResponse } from '../../types/response'
import ErrorHandler from '../../middleware/errorHandler'
import { upload } from '../../utils/upload'
import { validate } from '../../utils/validator'
import { uploadDocumentSchema } from './validation'
import type { UploadedFile } from './types'

const router = Router()

/**
 * POST /uploads
 * multipart/form-data example demonstrating JSON + array + file validation.
 *
 * Fields:
 *   title       string (required)
 *   tags[]      repeated string fields, or a JSON-encoded array (required)
 *   description string (optional)
 *   avatar      single image (PNG/JPEG, optional, max 1MB)
 *   documents   up to 3 additional files (optional, any type, max 5MB)
 */
router.post(
  '/',
  upload.fields([
    { name: 'avatar', maxCount: 1 },
    { name: 'documents', maxCount: 3 }
  ]),
  validate(uploadDocumentSchema),
  ErrorHandler.catchAsync(async (req: Request, res: Response) => {
    const { title, description, tags } = req.validated?.body ?? {}
    const files = req.validated?.files ?? {}

    const avatar = (files.avatar as unknown) as UploadedFile | undefined
    const documents = ((files.documents as unknown) || []) as UploadedFile[]

    const response = ApiResponse.created(
      {
        title,
        description,
        tags: tags ?? [],
        avatar: avatar ? { name: avatar.originalname, type: avatar.mimetype, size: avatar.size } : null,
        documents: documents.map((doc) => ({ name: doc.originalname, type: doc.mimetype, size: doc.size }))
      },
      'Upload validated and received'
    )
    res.status(201).json(response)
  })
)

export default router