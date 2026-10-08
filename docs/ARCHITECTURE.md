# Architecture map

Which crate is responsible for what, and which systems it is connected to. Arrows point from the crate that uses to the crate (or system) that is used. The rules behind it are enforced by `crates/architecture-tests`. Source of truth for the design is `docs/SPEC.md`; this page is the picture of what exists today.

```mermaid
flowchart TB
  subgraph external["Outside systems"]
    reaper["REAPER (DAW)<br/>transport, regions, markers, tempo map,<br/>project ExtState"]
    webview["Webview<br/>(Svelte 5 UI in the Tauri window)"]
    files["RC2/endpoint.json<br/>port + token on disk"]
    midi["MIDI devices<br/>(not built yet)"]
  end

  subgraph extension_side["Runs inside REAPER"]
    ext["reaper-extension<br/>cdylib: timer loop, safe mode, watchdog,<br/>builds the catalog, runs commands<br/>(the extension runs the show)"]
  end

  subgraph app_side["Runs on the stage computer"]
    app["app<br/>Tauri shell, composition root:<br/>link connection, command dispatch"]
  end

  subgraph wire["Link (loopback socket 127.0.0.1 + token)"]
    link["link<br/>LinkServer / LinkClient, hub,<br/>replay log, handshake"]
    protocol["protocol<br/>wire messages: State, Catalog, Command,<br/>LinkView; generates the TypeScript types"]
  end

  subgraph logic["Pure logic (no I/O, no REAPER, no clock)"]
    performance["performance<br/>state machine: phases, hand-over,<br/>hard stop, count-in"]
    catalogue["catalogue<br/>songs, cues, directives<br/>(!1008, !length, !bpm)"]
    setlists["setlists<br/>setlists and entries"]
    projections["projections<br/>joins the three above:<br/>PlannedSong list, views, live feed"]
    kernel["shared-kernel<br/>Seconds, Bpm, SongId, Id types"]
  end

  port["reaper-port<br/>ReaperPort trait, FakeReaper,<br/>scenario runner (tests)"]
  timerloop["timer-loop<br/>one step of REAPER's timer:<br/>performance, catalog, wire events"]
  arch["architecture-tests<br/>enforces who may depend on whom"]

  ext -- "reaper-rs (FFI)" --> reaper
  ext -- implements --> port
  ext --> timerloop & protocol & link
  timerloop --> port & performance & catalogue & setlists & protocol
  app -- "simulator (RC2_SIMULATOR)" --> timerloop
  ext -- writes --> files
  link --> protocol
  app --> link & protocol
  app -- reads --> files
  app <-- "Tauri events / invoke" --> webview
  link <-. "TCP, newline JSON" .-> link
  app == "push State + Catalog,<br/>receive Commands" ==> ext
  port --> performance
  projections --> performance & catalogue & setlists
  performance & catalogue & setlists & protocol --> kernel
  midi -.-> app
  arch -. checks .-> logic
```

Simplified data flow while playing:

1. REAPER plays. The extension's timer loop (about 30 Hz) reads the position, steps `performance`, and carries out the effects on REAPER through the `ReaperPort`.
2. When the state or the project content changes, the extension pushes `State` and `Catalog` over the link.
3. The app keeps a `LinkView`, emits it to the UI, and the UI draws it. The UI never works out state itself.
4. A button, key or (later) MIDI note becomes a `Command`, goes through `dispatch`, over the link, into a queue that the timer loop drains on its next tick. A connection thread never touches REAPER.

| Crate | Responsible for | Talks to |
|---|---|---|
| `shared-kernel` | value types (`Seconds`, `Bpm`, ids) | nothing |
| `catalogue` | songs, cues, marker directives | `shared-kernel` |
| `setlists` | setlist and entry rules | `shared-kernel` |
| `performance` | the one implementation of the performance rules | `shared-kernel` |
| `projections` | turning songs and a setlist into the planned list and views | `performance`, `catalogue`, `setlists` |
| `reaper-port` | the interface to REAPER and a fake for tests | `performance` |
| `protocol` | what goes over the link, and the generated TypeScript types | `shared-kernel` |
| `link` | the socket: server, client, replay | `protocol` |
| `timer-loop` | one step of REAPER's timer and what is pushed to the link, over any `ReaperPort` (ADR-012) | `performance`, `catalogue`, `setlists`, `protocol`, `link`, `reaper-port` |
| `reaper-extension` | everything that lives inside REAPER | REAPER, `timer-loop`, `link`, `protocol` |
| `app` | the stage app shell, and the simulator (the timer loop over `FakeReaper`) | `link`, `protocol`, `timer-loop`, `reaper-port`, the webview, `endpoint.json` |
| `architecture-tests` | the dependency rules | the manifests |
