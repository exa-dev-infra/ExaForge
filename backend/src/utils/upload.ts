import multer from 'multer'

export const MAX_FILE_SIZE = 5 * 1024 * 1024
export const MAX_FILES = 10

/**
 * In-memory multer configuration. Files are kept in RAM as Buffers so they can
 * be validated by the `validate()` middleware before being persisted.
 *
 * Mount it with `.fields([...])` on a route, or use the convenience helpers:
 *   import {uploads, singleFile} from '../utils/upload'
 *   router.post('/avatar', upload.single('avatar'), handler)
 *   router.post('/', upload.array('files', 5), handler)
 */
export const upload = multer({
  storage: multer.memoryStorage(),
  limits: {
    fileSize: MAX_FILE_SIZE,
    files: MAX_FILES
  }
})

export const uploadSingle = (fieldName: string) => upload.single(fieldName)

export const uploadMultiple = (fieldName: string, count = MAX_FILES) => upload.array(fieldName, count)

export default upload