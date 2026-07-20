#!/bin/bash
set -euo pipefail

echo "Starting USB Station development environment..."

# Start backend
echo "Building and starting backend..."
cd backend
cargo run &
BACKEND_PID=$!
cd ..

# Start frontend
echo "Installing frontend dependencies and starting dev server..."
cd frontend
if [ ! -d node_modules ]; then
  npm install
fi
npm run dev &
FRONTEND_PID=$!
cd ..

echo ""
echo "USB Station running:"
echo "  Frontend: http://localhost:5173"
echo "  Backend:  http://localhost:8080"
echo ""
echo "Press Ctrl+C to stop both services."

trap "kill $BACKEND_PID $FRONTEND_PID 2>/dev/null" EXIT
wait
