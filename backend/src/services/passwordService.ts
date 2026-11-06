import bcrypt from 'bcrypt'

class PasswordService {
  private saltRounds: number = 10

  /**
     * Hash password
     */
  async hash (password: string): Promise<string> {
    return await bcrypt.hash(password, this.saltRounds)
  }

  /**
     * Compare password with hash
     */
  async compare (password: string, hash: string): Promise<boolean> {
    return await bcrypt.compare(password, hash)
  }

  /**
     * Validate password strength
     */
  validateStrength (password: string): { valid: boolean; errors: string[] } {
    const errors: string[] = []

    if (password.length < 8) {
      errors.push('Password must be at least 8 characters long')
    }

    if (!/[A-Z]/.test(password)) {
      errors.push('Password must contain at least one uppercase letter')
    }

    if (!/[a-z]/.test(password)) {
      errors.push('Password must contain at least one lowercase letter')
    }

    if (!/[0-9]/.test(password)) {
      errors.push('Password must contain at least one number')
    }

    if (!/[!@#$%^&*(),.?":{}|<>]/.test(password)) {
      errors.push('Password must contain at least one special character')
    }

    return {
      valid: errors.length === 0,
      errors
    }
  }
}

export default new PasswordService()
