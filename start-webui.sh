#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

echo "Building Cogtome..."
cargo build --release

echo ""
echo "Starting API server on port 3334..."
"$SCRIPT_DIR/target/release/cogtome" serve --port 3334 &
API_PID=$!

# Wait for API to be ready
echo -n "Waiting for API server"
for i in {1..30}; do
    if curl -s http://localhost:3334/health > /dev/null 2>&1; then
        echo " OK"
        break
    fi
    echo -n "."
    sleep 0.5
done

echo ""
echo "Starting WebUI on port 3333..."
cd "$SCRIPT_DIR/webui" && npm run dev &
WEBUI_PID=$!

echo ""
echo "=========================================="
echo "  Cogtome WebUI is running!"
echo "  Frontend: http://localhost:3333"
echo "  API:      http://localhost:3334"
echo "=========================================="
echo ""
echo "Press Ctrl+C to stop all services"

cleanup() {
    echo ""
    echo "Stopping services..."
    kill $API_PID 2>/dev/null || true
    kill $WEBUI_PID 2>/dev/null || true
    exit 0
}

trap cleanup SIGINT SIGTERM

wait
