@echo off
echo ========================================
echo   TypeScript Project Setup Script
echo ========================================
echo.

echo [1/5] Installing dependencies...
call npm install
if errorlevel 1 (
    echo ERROR: Failed to install dependencies
    pause
    exit /b 1
)
echo ✓ Dependencies installed successfully
echo.

echo [2/5] Starting Docker services...
call npm run docker:up
if errorlevel 1 (
    echo ERROR: Failed to start Docker services
    echo Please make sure Docker is installed and running
    pause
    exit /b 1
)
echo ✓ Docker services started
echo.

echo [3/5] Waiting for services to be ready...
timeout /t 10 /nobreak > nul
echo ✓ Services should be ready
echo.

echo [4/5] Running database migrations...
call npm run migrate:up
if errorlevel 1 (
    echo ERROR: Failed to run migrations
    echo Please check your database connection
    pause
    exit /b 1
)
echo ✓ Migrations completed
echo.

echo [5/5] Building TypeScript...
call npm run build
if errorlevel 1 (
    echo ERROR: Failed to build TypeScript
    pause
    exit /b 1
)
echo ✓ Build completed
echo.

echo ========================================
echo   Setup Complete! 🎉
echo ========================================
echo.
echo Next steps:
echo   1. Start dev server:  npm run dev
echo   2. Run workers:       npm run workers
echo   3. Test API:          http://localhost:3000/health
echo.
echo Services running:
echo   - PostgreSQL:  localhost:5432
echo   - Redis:       localhost:6379
echo   - RabbitMQ:    localhost:5672
echo   - RabbitMQ UI: http://localhost:15672 (guest/guest)
echo.
echo For more info, check README.md and QUICKSTART.md
echo.
pause
