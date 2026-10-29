#!/usr/bin/env python3
"""Exercise dashboard login, tenant isolation, and WebSocket revocation.

Test dependencies only: httpx==0.28.1 websockets==15.0.1.
The server uses a disposable SQLite database and never reads personal prompts.
"""
import argparse
import asyncio
import json
import os
from pathlib import Path
import socket
import subprocess
import tempfile

import httpx
import websockets


async def smoke(binary):
    with tempfile.TemporaryDirectory(prefix="sovereign-dashboard-test-") as directory:
        with socket.socket() as port_socket:
            port_socket.bind(("127.0.0.1", 0))
            port = port_socket.getsockname()[1]
        base = f"http://127.0.0.1:{port}"
        env = {k: v for k, v in os.environ.items() if not k.startswith("SOVEREIGN_") and k != "DATABASE_URL"}
        env.update(SOVEREIGN_DASHBOARD_ONLY="true", SOVEREIGN_DASHBOARD_ADDR=f"127.0.0.1:{port}",
                   SOVEREIGN_DB_PATH=str(Path(directory) / "test.db"),
                   SOVEREIGN_API_KEYS=json.dumps({"alice": "a" * 32, "bob": "b" * 32}))
        with open(Path(directory) / "server.log", "w+") as log:
            process = subprocess.Popen([str(binary.resolve())], cwd=directory, env=env, stdout=log, stderr=log)
            try:
                async with httpx.AsyncClient(base_url=base) as client:
                    for _ in range(100):
                        if process.poll() is not None:
                            log.seek(0)
                            raise RuntimeError("Dashboard startup failed: " + log.read())
                        try:
                            if (await client.get("/health")).status_code == 200:
                                break
                        except httpx.ConnectError:
                            pass
                        await asyncio.sleep(0.1)
                    else:
                        raise TimeoutError("Dashboard startup")
                    assert (await client.get("/api/history/alice")).status_code == 401
                    assert (await client.post("/api/login", json={"token": "invalid"})).status_code == 401
                    response = await client.post("/api/login", json={"token": "a" * 32})
                    assert response.status_code == 200
                    cookie = response.headers["set-cookie"].split(";", 1)[0]
                    assert "a" * 32 not in cookie
                    assert "HttpOnly" in response.headers["set-cookie"]
                    assert (await client.get("/api/session")).json()["user_id"] == "alice"
                    assert (await client.get("/api/history/alice")).status_code == 200
                    assert (await client.get("/api/history/bob")).status_code == 403
                    async with websockets.connect(f"ws://127.0.0.1:{port}/ws/analytics/alice",
                                                  additional_headers={"Cookie": cookie}, origin=base) as stream:
                        assert json.loads(await stream.recv())["user_id"] == "alice"
                        assert (await client.post("/api/logout")).status_code == 204
                        try:
                            async with asyncio.timeout(6):
                                while True:
                                    await stream.recv()
                        except websockets.ConnectionClosed:
                            pass
                        else:
                            raise AssertionError("Logout did not revoke the open WebSocket")
                    assert (await client.get("/api/history/alice", headers={"Cookie": cookie})).status_code == 401
                return {"ok": True, "login": True, "tenant_isolation": True,
                        "websocket_logout_revocation": True, "personal_database_opened": False}
            finally:
                process.terminate()
                try:
                    process.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait(timeout=5)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    args = parser.parse_args()
    print(json.dumps(asyncio.run(asyncio.wait_for(smoke(args.binary), 60)), indent=2))
