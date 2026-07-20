#!/bin/bash
set -euo pipefail

echo "USB Station Setup"
echo "================"

# Check prerequisites
command -v rustc >/dev/null 2>&1 || { echo "Error: Rust is required. Install from https://rustup.rs"; exit 1; }
command -v node >/dev/null 2>&1 || { echo "Error: Node.js is required."; exit 1; }
command -v docker >/dev/null 2>&1 || { echo "Warning: Docker not found. Only local dev will work."; }

# Create data directories
mkdir -p backend/data/iso backend/data/downloads backend/data/logs

# Install backend
echo "Building backend..."
cd backend
cargo build --release
cd ..

# Install frontend
echo "Installing frontend dependencies..."
cd frontend
npm install
cd ..

echo ""
echo "Setup complete!"
echo ""
echo "To run locally:"
echo "  cp backend/.env.example backend/.env"
echo "  ./scripts/dev.sh"
echo ""
echo "To run with Docker:"
echo "  docker compose -f docker/docker-compose.yml up --build"
