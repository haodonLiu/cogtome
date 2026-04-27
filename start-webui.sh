#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

check_port() {
    if lsof -i:$1 > /dev/null 2>&1; then
        echo -e "${RED}Error: Port $1 is already in use${NC}"
        return 1
    fi
    return 0
}

echo -e "${GREEN}Building Cogtome backend...${NC}"
cargo build --release

echo -e "${GREEN}Building WebUI...${NC}"
cd "$SCRIPT_DIR/webui"
npm run build

echo ""
echo -e "${GREEN}Starting API server on port 3334...${NC}"
cd "$SCRIPT_DIR"
"$SCRIPT_DIR/target/release/cogtome" serve --port 3334 &
API_PID=$!

# Wait for API to be ready
echo -n "Waiting for API server"
for i in {1..30}; do
    if curl -s http://localhost:3334/health > /dev/null 2>&1; then
        echo -e " ${GREEN}OK${NC}"
        break
    fi
    echo -n "."
    sleep 0.5
done

echo ""
echo -e "${GREEN}Starting WebUI on port 3333...${NC}"
cd "$SCRIPT_DIR/webui" && npm run dev &
WEBUI_PID=$!

echo ""
echo "=========================================="
echo -e "  ${GREEN}Cogtome WebUI is running!${NC}"
echo "  Frontend: ${YELLOW}http://localhost:3333${NC}"
echo "  API:      ${YELLOW}http://localhost:3334${NC}"
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
