export interface UploadedFile {
  originalname: string
  mimetype: string
  size: number
}

export interface UploadResponsePayload {
  title?: string
  description?: string
  tags: unknown[]
  avatar: {name: string; type: string; size: number} | null
  documents: UploadedFile[]
}