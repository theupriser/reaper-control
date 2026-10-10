# Security review (WP 7.4, macOS, 2026-10-10)

Scope: what another program or person on the same machine, or on the same network, can do to the extension and the app. Method: read the code, then look at the running isolated REAPER from outside with `scripts/security_check.sh`.

## Findings

| # | Finding | Result |
|---|---------|--------|
| 1 | The extension listens on `127.0.0.1` only (`TcpListener::bind((Ipv4Addr::LOCALHOST, 0))`). | Seen: `lsof` shows one listener, `127.0.0.1:<port>`; the machine's LAN address refuses the port. |
| 2 | A peer must send a valid Hello with the 256-bit token first; the token is compared without stopping at the first difference. Wrong token, wrong protocol, junk bytes, a 4 GiB length and a first message that is not a Hello are all closed without a reply. | Seen live (script) and in `link/tests/link.rs`. |
| 3 | Frames are capped at 1 MiB; the decoder never allocates for a larger length; the parser is fuzzed in CI. | Covered. |
| 4 | **`endpoint.json` (it holds the token) was created with the default mode `0644`, readable by every account on the machine.** | **Fixed**: it is written with mode `0600` (unix). Test `the_endpoint_file_is_readable_only_by_its_owner`; seen live: `-rw-------`. On Windows the file inherits the user profile's access list (REAPER's resource folder is under the user's profile); not checked on a real Windows machine. |
| 5 | **The app window had no Content Security Policy (`csp: null`).** | **Fixed**: `default-src 'self'`, no remote scripts, images or frames, `connect-src` only Tauri's IPC, no objects, no base tag, no forms. A separate `devCsp` lets the Vite dev server work. The release app starts and renders with it (window screenshot, Player screen, "Connected to REAPER"). |
| 6 | The Tauri capability for the window is `core:default` only. No shell, filesystem, opener or HTTP plugin is installed, and no remote URL is allowed. | Read; nothing to change. |
| 7 | App commands take no paths or commands from the UI: diagnostics go to a fixed folder, a setlist import takes ids that must exist in the project mirror, settings are validated and refused when invalid (`AppConfig::with_settings`). | Read; nothing to change. |
| 8 | At most 16 connections; a peer that stays silent is dropped after 5 s. | Accepted: a program on the same machine that opens 16 silent connections can block a real client for 5 s at a time. The same program could read the token's file or kill the process if it ran as the same user, so this is not closed further. |

## What this does not cover

- A program running as the same user can read the token and control REAPER; that is the model (SPEC S-9 does not defend against it).
- Windows: firewall prompt, file access list of `endpoint.json`.
- Signing and notarisation (S4), the update path (v2.1), a review by someone else.
