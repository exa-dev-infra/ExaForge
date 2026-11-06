export interface RegisterDTO {
  name: string
  email: string
  password: string
  role?: string
}

export interface LoginDTO {
  email: string
  password: string
}

export interface AuthUser {
  id: number
  name: string
  email: string
  role: string
}

export interface TokenPair {
  accessToken: string
  refreshToken: string
}

export interface AuthResponse {
  user: AuthUser
  tokens: TokenPair
}

export interface RefreshTokenResponse {
  user: AuthUser
  tokens: TokenPair
  rotated: boolean
}