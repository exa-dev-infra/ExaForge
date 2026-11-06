import type { Request, Response } from 'express'
import { Router } from 'express'
import { ApiResponse } from '../../types/response'
import ErrorHandler from '../../middleware/errorHandler'
import { authenticate, authorize } from '../../middleware/auth'
import { validate } from '../../utils/validator'
import userService from './service'
import { createUserSchema, listUsersSchema, updateUserSchema, userIdSchema } from './validation'

const router = Router()

// All user management routes require authenticated admin access
router.use(authenticate)
router.use(authorize('admin'))

// GET all users (optional pagination)
router.get(
  '/',
  validate(listUsersSchema),
  ErrorHandler.catchAsync(async (req: Request, res: Response) => {
    const { page, limit } = req.validated?.query ?? {}

    if (page && limit) {
      const result = await userService.getUsersPaginated(Number(page), Number(limit))
      const response = ApiResponse.paginated(
        result.data,
        result.page,
        result.limit,
        result.total,
        'Users retrieved successfully'
      )
      res.json(response)
      return
    }

    const users = await userService.getAllUsers()
    const response = ApiResponse.success(users, 'Users retrieved successfully')
    res.json(response)
  })
)

// GET user by ID
router.get(
  '/:id',
  validate(userIdSchema),
  ErrorHandler.catchAsync(async (req: Request, res: Response) => {
    const user = await userService.getUserById(String(req.params.id))

    const response = ApiResponse.success(user, 'User retrieved successfully')
    res.json(response)
  })
)

// CREATE new user (Admin)
router.post(
  '/',
  validate(createUserSchema),
  ErrorHandler.catchAsync(async (req: Request, res: Response) => {
    const user = await userService.createUser(
      req.validated?.body as unknown as Parameters<typeof userService.createUser>[0]
    )

    const response = ApiResponse.created(user, 'User created successfully')
    res.status(201).json(response)
  })
)

// UPDATE user (Admin)
router.put(
  '/:id',
  validate(updateUserSchema),
  ErrorHandler.catchAsync(async (req: Request, res: Response) => {
    const user = await userService.updateUser(
      String(req.params.id),
      req.validated?.body as unknown as Parameters<typeof userService.updateUser>[1]
    )

    const response = ApiResponse.success(user, 'User updated successfully')
    res.json(response)
  })
)

// DELETE user (Admin - Hard delete with atomic token revocation)
router.delete(
  '/:id',
  validate(userIdSchema),
  ErrorHandler.catchAsync(async (req: Request, res: Response) => {
    await userService.deleteUser(String(req.params.id))

    const response = ApiResponse.deleted('User deleted successfully')
    res.json(response)
  })
)

export default router