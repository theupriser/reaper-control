#!/usr/bin/env bash
# Looks at the running extension from outside, the way another program on the machine would:
# where it listens, who can read the token, and what it does with a wrong token and with junk.
# Needs the isolated REAPER to be running with the extension (see scripts/soak.sh for the start).
set -euo pipefail
cd "$(dirname "$0")/.."
R="$PWD/.dev/reaper-test"
ENDPOINT=$R/RC2/endpoint.json
[ -f "$ENDPOINT" ] || { echo "no $ENDPOINT: start the isolated REAPER first"; exit 1; }
PID=$(pgrep -f "REAPER.*-cfgfile $R/reaper.ini" | head -1)
PORT=$(python3 -I -c "import json,sys; print(json.load(open(sys.argv[1]))['port'])" "$ENDPOINT")

echo "== Listening sockets of REAPER (pid $PID)"
lsof -nP -a -p "$PID" -iTCP -sTCP:LISTEN
echo "== Endpoint file"
ls -l "$ENDPOINT"
python3 -I - "$PORT" <<'PY'
import socket, struct, sys, json
port = int(sys.argv[1])

def attempt(name, payload):
    s = socket.create_connection(("127.0.0.1", port), timeout=3)
    s.sendall(payload)
    try:
        reply = s.recv(4096)
    except socket.timeout:
        reply = b"(no reply, still open)"
    print(f"   {name}: {'closed without a reply' if reply == b'' else reply[:60]}")

def frame(body):
    return struct.pack(">I", len(body)) + body

hello = lambda token, protocol: json.dumps({"type": "Hello", "protocol": protocol, "token": token, "resume_from_event_id": None}).encode()
print("== Hostile peers (each must be closed without a reply)")
attempt("wrong token", frame(hello("0" * 64, 3)))
attempt("empty token", frame(hello("", 3)))
attempt("junk bytes", b"GET / HTTP/1.1\r\n\r\n")
attempt("oversized length (4 GiB)", struct.pack(">I", 0xFFFFFFFF))
attempt("not a Hello first", frame(json.dumps({"type": "GetCatalog"}).encode()))
print("== Remote reachability")
try:
    import subprocess
    addr = subprocess.check_output(["ipconfig", "getifaddr", "en0"], text=True).strip()
    socket.create_connection((addr, port), timeout=2)
    print(f"   FAIL: {addr}:{port} accepted a connection")
    sys.exit(1)
except (OSError, subprocess.CalledProcessError) as error:
    print(f"   LAN address refused or absent ({type(error).__name__}): ok")
PY
echo "security check: ok"
