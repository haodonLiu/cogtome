#!/usr/bin/env python3
"""
Python Motif Server - receives execution requests via Unix socket

Usage:
    COGTOME_SOCKET_PATH=/tmp/cogtome/motifs/my-motif.sock python3 python_server.py

Request format (JSON-RPC 2.0):
{
    "jsonrpc": "2.0",
    "method": "execute",
    "params": {
        "motif_name": "my-motif",
        "input": {...}
    },
    "id": 1
}

Response format:
{
    "jsonrpc": "2.0",
    "result": {...},
    "id": 1
}
"""

import asyncio
import json
import os
import sys
from pathlib import Path

# Motifs registry: motif_name -> callable
MOTIFS = {}


def register_motif(name: str):
    """Decorator to register a motif function"""
    def decorator(func):
        MOTIFS[name] = func
        return func
    return decorator


# Example motifs
@register_motif("hello")
async def hello_motif(input_data: dict) -> dict:
    """Simple greeting motif"""
    name = input_data.get("name", "World")
    return {"greeting": f"Hello, {name}!"}


@register_motif("transform")
async def transform_motif(input_data: dict) -> dict:
    """Text transformation motif"""
    text = input_data.get("text", "")
    operations = input_data.get("ops", [])

    result = text
    for op in operations:
        if op == "uppercase":
            result = result.upper()
        elif op == "lowercase":
            result = result.lower()
        elif op == "reverse":
            result = result[::-1]
        elif op == "trim":
            result = result.strip()

    return {"result": result}


class MotifServer:
    def __init__(self, socket_path: str):
        self.socket_path = Path(socket_path)
        self.running = True

    async def handle(self, reader: asyncio.StreamReader, writer: asyncio.StreamWriter):
        addr = writer.get_extra_info('peername')
        print(f"[Python Motif Server] Connected from {addr}")

        try:
            while self.running:
                line = await reader.readline()
                if not line:
                    break

                try:
                    request = json.loads(line.decode())
                    response = await self.process_request(request)
                except json.JSONDecodeError as e:
                    response = {
                        "jsonrpc": "2.0",
                        "error": {"code": -32700, "message": f"Parse error: {e}"},
                        "id": None
                    }
                except Exception as e:
                    response = {
                        "jsonrpc": "2.0",
                        "error": {"code": -32603, "message": f"Internal error: {e}"},
                        "id": request.get("id") if 'request' in dir() else None
                    }

                response_line = json.dumps(response) + "\n"
                writer.write(response_line.encode())
                await writer.drain()

        except Exception as e:
            print(f"[Python Motif Server] Error: {e}")
        finally:
            writer.close()
            await writer.wait_closed()
            print(f"[Python Motif Server] Disconnected from {addr}")

    async def process_request(self, request: dict) -> dict:
        method = request.get("method")
        params = request.get("params", {})
        request_id = request.get("id")

        if method == "execute":
            motif_name = params.get("motif_name")
            input_data = params.get("input", {})

            if motif_name not in MOTIFS:
                return {
                    "jsonrpc": "2.0",
                    "error": {"code": -32601, "message": f"Motif '{motif_name}' not found"},
                    "id": request_id
                }

            try:
                motif_func = MOTIFS[motif_name]
                result = await motif_func(input_data)
                return {
                    "jsonrpc": "2.0",
                    "result": result,
                    "id": request_id
                }
            except Exception as e:
                return {
                    "jsonrpc": "2.0",
                    "error": {"code": -32603, "message": f"Motif execution error: {e}"},
                    "id": request_id
                }

        elif method == "list_motifs":
            return {
                "jsonrpc": "2.0",
                "result": {"motifs": list(MOTIFS.keys())},
                "id": request_id
            }

        elif method == "shutdown":
            self.running = False
            return {
                "jsonrpc": "2.0",
                "result": {"status": "shutdown"},
                "id": request_id
            }

        else:
            return {
                "jsonrpc": "2.0",
                "error": {"code": -32601, "message": f"Method '{method}' not found"},
                "id": request_id
            }


async def main():
    socket_path = os.environ.get("COGTOME_SOCKET_PATH")
    if not socket_path:
        print("[Python Motif Server] Error: COGTOME_SOCKET_PATH not set")
        sys.exit(1)

    socket_path = Path(socket_path)

    # Remove existing socket file
    if socket_path.exists():
        socket_path.unlink()

    # Ensure parent directory exists
    socket_path.parent.mkdir(parents=True, exist_ok=True)

    print(f"[Python Motif Server] Starting on {socket_path}")
    print(f"[Python Motif Server] Registered motifs: {list(MOTIFS.keys())}")

    server = MotifServer(str(socket_path))
    async with await asyncio.start_unix_server(server.handle, str(socket_path)) as server:
        print("[Python Motif Server] Ready and listening")
        await asyncio.Event().wait()


if __name__ == "__main__":
    asyncio.run(main())
