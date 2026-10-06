# Review 1 — consistency & design check of SPEC.md / PLAN.md

Status legend: **FIXED** (edited in docs) · **DECISION** (needs owner choice) · **SPIKE** (resolved by measurement).

## A. Design faults (need decisions)
| # | Finding | Why it matters | Proposed resolution | Status |
|---|---------|----------------|---------------------|--------|
| A1 | **Two authorities for playback.** Script owns playback (§2.1) but §14 defines a Rust `Performance` aggregate that "loads, behaves, publishes events", and Phase 2 builds it *and* Phase 3 ports it to Lua. Same for the marker parser (§4) and count-in maths. This contradicts goal 4 (one state machine, one parser) and risks split-brain. | Core stage-safety/maintainability | **Write the Performance rules once, in Lua (pure `core/`, no `reaper.*` calls), and host that same code in Rust through `mlua` for the simulator, FakeDriver, dry-run and tests.** Rust keeps the *read side* (projections, intent validation, Catalogue/Setlists editing rules). Cross-testing (WP 3.2) disappears. Cost: Lua becomes the domain language for the core; `mlua` build on 3 OS. Alternative: Rust is authoritative and a native REAPER extension (reaper-rs) replaces the Lua script (bigger, loses easy install). | **DECIDED: D5 (Lua once, mlua host)** — SPEC §3/§14 and PLAN Phases 2–3 rewritten |
| A2 | **Hand-over timing is not sample-accurate.** Earlier claims ("no latency guard") ignore the ~33 ms tick of a defer loop; also the loop may stall while REAPER shows modal UI. | Missed/late hand-overs live | Lead = tick+seek (measured), `handOverLeadMs` advanced setting; S2 to test **native region playlists / smooth seek**, which would remove the dependency on the script loop entirely (hard stops/count-in stay scripted). | SPIKE (S2, S3) — text FIXED |
| A3 | **Event loss.** Polling a state snapshot can miss transient events (hand-over between polls); journal in the app is useless if the app is dead. | S-8, correctness of domain events | Script publishes an **event ring** and writes the **journal itself**. | FIXED in SPEC §2.2/S-8 |
| A4 | **Payload design.** One JSON blob with regions/markers/setlists at 10–20 Hz. | Latency, CPU, goal 5 | Split into `live` (small, hot) and `catalog` (cold, revision-gated). | FIXED |
| A5 | **Single command slot** overwritable between ticks; ack-per-command serialises latency. | Lost presses on stage | Ordered mailbox batch + id dedupe + highest-id ack. | FIXED |
| A6 | **Song identity.** Setlists reference region ids; REAPER region numbers can change; ProjectId derivation unclear (v1 import depends on it). | Setlists silently break | Spike S6, ADR-008; validation/repair UI. | SPIKE (S6, new) |
| A7 | **Web interface is open to the network** (any host can send `_/1007`). Venue Wi-Fi + public release. | Safety/security | Installer restricts access (password/localhost) — verify in S4; remote (D4) uses own authenticated channel. | FIXED in text; SPIKE S4 |
| A8 | **Link health cannot distinguish** "REAPER closed" from "web interface off" from "script missing". | Wrong guidance in wizard/banner | Cause classification, using Installation's process check. | FIXED |
| A9 | **Schedule estimate wrong:** WPs sum to ≈197 d, not 120–145 d. | Planning | See scope options below. | FIXED (number) / **DECISION** (scope) |
| A10 | **Reducer vs aggregate vs state machine**: §11 (reducer), §14 (aggregates), §3 (state machine) describe three mutation models. | Confusion, duplicated concepts | With A1: write side = Lua state machine; app side = reducers/projections only; aggregates only in Setlists/Installation/Link where the app is authoritative. §14.3/14.4 rewritten. | FIXED |

### Scope options for A9 (pick any)
1. Ship v2.0 on macOS+Windows only; Linux after (−6 d QA/signing, lowers R5).
2. Updater and website after 2.0 (−5 d).
3. Skip i18n/NL, MIDI learn, hold-to-confirm in 2.0 (−5 d).
4. ~~Replace Rust Performance aggregate by Lua-once~~ — adopted as D5; net effect on total was ≈ 0 (host + journal + event ring cost what the Rust aggregate saved), the gain is correctness, not days.
Realistic 2.0 after cuts 1–3 ≈ 180 d.

## B. Contradictions fixed
| # | Contradiction | Fix |
|---|---------------|-----|
| B1 | "No new features" vs installer, checklist, simulator, learn mode, hold-to-confirm | Explicit *parity definition* + allowed additions list (SPEC §0) |
| B2 | Non-goals excluded remote clients while D4 adds them | Reworded: stretch only |
| B3 | Phase names differ in §2.3 / §3 / §14 (`Transitioning` vs `HandingOver`, `AwaitingStart` vs `CountingIn`, no `Paused`) | One vocabulary: `Idle, Playing, Paused, CountingIn, HardStopped, HandingOver, Finished` |
| B4 | Glossary bans "item/playlist/transition" but §2.3/§2.4/S-8 use them | State model + commands renamed (`entries`, `performance`, `SeekSong`) |
| B5 | Count-in rule contradicted v1 (v1 counts in only on marker jumps) | F10 + rule 5/6 clarified |
| B6 | §2.1 pointed to wizard in "§12" (decisions log) | → §13 |
| B7 | Two crate layouts (PLAN §3 tree, §11 hexagonal vs §14.5 per-context) and two ADR paths | PLAN tree and SPEC rule updated, ADR path `docs/adr/` |
| B8 | Two schedules (SPEC §8 milestones vs PLAN phases) | SPEC §8 now points to PLAN |
| B9 | `TransitionStrategy` as a config option vs anti-goal "no abstraction without two implementations" | Decided by ADR-005, not a setting |
| B10 | Parity items silently dropped/unclear (timeline click-to-seek, totals in performer, system stats, transition offset) | F11, F13, F14 clarified, F18 added |
| B11 | Setlist authority (script persists, Rust aggregate edits, mirror backup) unclear | `SaveSetlist(expectedRev)` single write path; mirror is restore-only |
| B12 | S-6 "everything via MIDI" vs wizard/settings | Limited to performance actions |

## C. Still to do in docs (after A1)
- Rewrite SPEC §14.3/§14.4 per the A1 outcome; update PLAN Phases 2–3 (Lua core + mlua host instead of Rust aggregate + port).
- Add missing work packages: system stats (`sysinfo`, 1 d), script-side journal (1 d), event ring (1 d), web-interface hardening in installer (1 d), `mlua` host (3 d, if A1 accepted).
- Reorder WP numbering cosmetically; Gate criteria for Phase 5/6 to reference F1–F18.

## D. Update 2026-10-06 (decision D7)
A1 was resolved with D5 (Lua once, `mlua` host). That was then superseded by **D7**: the primary design is a native Rust REAPER extension (`reaper-rs`), so the performance rules are written once in Rust and shared by extension and app; D5 remains only as the fallback (SPEC §2.5). A2 (hand-over timing) and the open web-interface security point (A7) are now handled by S2 and by the local-only socket. New risk class: extension crash (S-9, PLAN R15).
