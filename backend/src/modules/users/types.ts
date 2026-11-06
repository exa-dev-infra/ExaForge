export interface User {
  id: number
  name: string
  email: string
  password?: string
  role?: string
  created_at?: Date
  updated_at?: Date
}

export interface CreateUserDTO {
  name: string
  email: string
  password?: string
  role?: string
}

export interface UpdateUserDTO {
  name?: string
  email?: string
  password?: string
  role?: string
}

export interface PaginatedUsers {
  data: User[]
  total: number
  page: number
  limit: number
  totalPages: number
}