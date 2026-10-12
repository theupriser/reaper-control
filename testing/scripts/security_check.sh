#!/usr/bin/env bash
# Looks at the running extension from outside, the way another program on the machine would:
# a wrong token, junk and an oversized length must each be closed without a reply, and the port
# must not be reachable on the machine's network address.
# Runs after smoke_extension.sh starts REAPER (it sets REAPER_RESOURCE). By hand, start the isolated
# REAPER first and set REAPER_RESOURCE to its resource directory (default .dev/reaper-test).
set -euo pipefail
cd "$(dirname "$0")/../.."

resource=${REAPER_RESOURCE:-$PWD/.dev/reaper-test}
endpoint=$resource/RC2/endpoint.json
[ -f "$endpoint" ] || { echo "no $endpoint: start the isolated REAPER first"; exit 1; }

case "$(uname -s)" in MINGW*|MSYS*|CYGWIN*) python=python ;; *) python=python3 ;; esac

echo "== Endpoint file"
ls -l "$endpoint"
"$python" -I - "$endpoint" <<'PY'
import json, socket, struct, sys

port = json.load(open(sys.argv[1]))["port"]
failures = []

def attempt(name, payload):
    connection = socket.create_connection(("127.0.0.1", port), timeout=3)
    connection.sendall(payload)
    try:
        reply = connection.recv(4096)
    except socket.timeout:
        reply = b"(no reply, still open)"
    except ConnectionError:
        reply = b""
    closed = reply == b""
    print(f"   {name}: {'closed without a reply' if closed else reply[:60]}")
    if not closed:
        failures.append(name)

def frame(body):
    return struct.pack(">I", len(body)) + body

def hello(token, protocol):
    return json.dumps({"type": "Hello", "protocol": protocol, "token": token, "resume_from_event_id": None}).encode()

print("== Hostile peers (each must be closed without a reply)")
attempt("wrong token", frame(hello("0" * 64, 3)))
attempt("empty token", frame(hello("", 3)))
attempt("junk bytes", b"GET / HTTP/1.1\r\n\r\n")
attempt("oversized length (4 GiB)", struct.pack(">I", 0xFFFFFFFF))
attempt("not a Hello first", frame(json.dumps({"type": "GetCatalog"}).encode()))

print("== Remote reachability")
probe = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
try:
    probe.connect(("192.0.2.1", 9))  # documentation address: nothing is sent, it only picks our network address
    address = probe.getsockname()[0]
except OSError:
    address = None
finally:
    probe.close()
if address is None or address.startswith("127."):
    print("   no network address on this machine: nothing to reach")
else:
    try:
        socket.create_connection((address, port), timeout=2).close()
        print(f"   FAIL: {address}:{port} accepted a connection")
        failures.append("reachable from the network")
    except OSError as error:
        print(f"   {address}:{port} refused or unreachable ({type(error).__name__}): ok")

if failures:
    print("security check: FAILED:", ", ".join(failures))
    sys.exit(1)
PY
echo "security check: ok"
