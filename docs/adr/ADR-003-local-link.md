# ADR-003: Link = loopback TCP, all socket work off the main thread

Status: **Proposed**. Spike S3 is positive on macOS arm64; Windows and several design points are untested (below).
Date: 2026-10-07. Related: ADR-002, SPEC §2.5 (link), S-9 (no blocking on main/audio thread), PLAN WP 1.3.

## Question (S3)
Can the extension push state to the app over a local socket without ever disturbing REAPER's main thread, even with slow, dead, hostile or flapping clients?

## What was run
- Spike crate `spikes/s3-socket` (outside the workspace): an extension plus a test client (`s3-client`: `watch`, `ping`, `slow`, `churn`, `badtoken`). REAPER 7.82, separate `-cfgfile` instance, macOS arm64.
- Extension: `TcpListener` on `127.0.0.1:0`; port and a random token written to `<resource>/RC2/s3-endpoint.txt`. First line from a client must be `HELLO <token>`, else the connection is closed without a reply.
- Threads: acceptor, one reader and one writer per client, one broadcaster. The main-thread tick only builds a 4 KB state line (position, play state, sequence number, timestamp) and calls `try_send` on a bounded channel; it also drains inbound commands with `try_recv`.
- Each client has a bounded queue (32 lines). A full queue, or a write error or 1 s write timeout, drops that client and shuts its socket down. Nobody is waited for.
- State is 4 KB per tick (about 135 KB/s) on purpose, so a stalled reader fills its buffers in seconds.

## Results (measured)
| Test | Result |
|---|---|
| One client, 10 s | 333 states, 0 missed (sequence numbers), push latency mean 0.29 ms, p99 0.86 ms, max 6.9 ms; interval 30.0 ms (23-36) |
| Ping through the main-thread tick, 200 pings | round trip mean 18.4 ms, max 32.4 ms: bounded by the 30 ms tick, as designed |
| Bad token | connection closed, 0 bytes read |
| Client never reads, 15-20 s, next to a healthy client | slow client dropped after its queue filled; the healthy client received every state (0 missed); tick unchanged (30.0 ms, max 34 ms) |
| Slow client afterwards | first run: connection stayed open (bug in the spike: the read half kept the socket alive). After the writer shuts the socket down: EOF seen by the client |
| `kill -9` on a connected client | cleaned up, clients back to 0 |
| 20 parallel clients x 25 connect/auth/close cycles, then 500 more sequentially | all succeeded (500 in 0.05 s); tick max 38.0 ms (min 22.0), `main_queue_full` stayed 0 |

Tick summary during all of this: average 30.0 ms. The only large values (129 ms, 70 ms) are at project load, as in S1.

## Proposed decision
- Link = loopback TCP (127.0.0.1, port from the OS, endpoint file with port and token). Auth by token on the first line.
- Main thread: `try_send` / `try_recv` only. All socket I/O on other threads. Per-client bounded queue; a slow client is dropped, never waited for. Shut the socket down when a client is dropped.
- Commands from the app are handled in the next tick (about 15 ms on average, 30 ms worst case at this tick rate).

## Not tested (do before Gate 1 or in the real implementation)
- Windows: only compiled by CI, never run (needs a machine). Named pipes / Unix sockets were not compared; TCP worked, so no reason to switch yet.
- Firewall or antivirus prompts for a loopback listener (macOS and Windows).
- Reconnect with replay (resume after a gap): protocol design, not part of this spike.
- Behaviour during REAPER menus and modal dialogs (same open item as ADR-005); sockets run on their own threads, so only the tick is at risk.
- Log writes from several threads interleave in the spike; the real extension needs one logging thread.
