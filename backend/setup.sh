#!/bin/bash

echo "========================================"
echo "   TypeScript Project Setup Script"
echo "========================================"
echo ""

echo "[1/5] Installing dependencies..."
npm install
if [ $? -ne 0 ]; then
    echo "ERROR: Failed to install dependencies"
    exit 1
fi
echo "✓ Dependencies installed successfully"
echo ""

echo "[2/5] Starting Docker services..."
npm run docker:up
if [ $? -ne 0 ]; then
    echo "ERROR: Failed to start Docker services"
    echo "Please make sure Docker is installed and running"
    exit 1
fi
echo "✓ Docker services started"
echo ""

echo "[3/5] Waiting for services to be ready..."
sleep 10
echo "✓ Services should be ready"
echo ""

echo "[4/5] Running database migrations..."
npm run migrate:up
if [ $? -ne 0 ]; then
    echo "ERROR: Failed to run migrations"
    echo "Please check your database connection"
    exit 1
fi
echo "✓ Migrations completed"
echo ""

echo "[5/5] Building TypeScript..."
npm run build
if [ $? -ne 0 ]; then
    echo "ERROR: Failed to build TypeScript"
    exit 1
fi
echo "✓ Build completed"
echo ""

echo "========================================"
echo "   Setup Complete! 🎉"
echo "========================================"
echo ""
echo "Next steps:"
echo "  1. Start dev server:  npm run dev"
echo "  2. Run workers:       npm run workers"
echo "  3. Test API:          http://localhost:3000/health"
echo ""
echo "Services running:"
echo "  - PostgreSQL:  localhost:5432"
echo "  - Redis:       localhost:6379"
echo "  - RabbitMQ:    localhost:5672"
echo "  - RabbitMQ UI: http://localhost:15672 (guest/guest)"
echo ""
echo "For more info, check README.md and QUICKSTART.md"
echo ""
