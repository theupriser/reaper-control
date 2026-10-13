# Testing

Everything the tests read or run that is not Rust or UI code.

```
testing/
  README.md
  data/
    projects/    sample REAPER projects (markers and regions, no audio) for trying things by hand and for the soak scripts
    reaper/      the starting config of the isolated REAPER: reaper.ini (no VST scan, default audio device, no save prompt) and windows.ini (Dummy Audio, added on Windows)
    scenarios/   performance scenarios (JSON), read by the reaper-port tests
    vectors/     protocol and marker test vectors (JSON), read by the protocol and catalogue tests
  scripts/       scripts that run against a real REAPER
    chaos/       project switches, live markers and a 1000-region project
```

- `data/` holds fixtures: what tests read, and sample REAPER projects. No scripts.
- `scripts/` holds scripts that start an isolated REAPER (a throwaway copy under the git-ignored `.dev/`, never your own REAPER). Each script starts with a header that says what it checks and what it needs. Run them from the repository root.

An architecture test fails when something else sits in `testing/`.

## Scripts

| Script | What it checks | Needs |
|---|---|---|
| `scripts/smoke_extension.sh` | the packaged extension loads and listens; runs further check scripts while REAPER is up | macOS or Windows, packaged extension (also runs in CI) |
| `scripts/security_check.sh` | a wrong token, junk and an oversized length are closed without a reply; the port is not reachable on the network address | run by the smoke script (also in CI) |
| `scripts/resource_audit.sh` | idle CPU and memory of the app, REAPER and the extension | macOS, release build |
| `scripts/autostart_check.sh` | a project with the autostart switch on gets the app started once (a stand-in program takes its place), a project without it starts nothing | run by the smoke script with `data/projects/autostart-sample.rpp` or `s1-sample.rpp` (also in CI) |
| `scripts/closing_check.sh` | a clean quit of REAPER sends the app a goodbye and removes the `running` file; a kill sends nothing and leaves the file | run by the smoke script, once with `CLOSING_MODE=quit` and once with `kill` (also in CI) |
| `scripts/soak_short.sh` | 8 songs of 6 s: every hand-over completes within 0.5 s of the plan (about a minute; with `soak_short_project.sh`) | run by the smoke script (also in CI) |
| `scripts/soak.sh` | 40 songs in a row: hand-over timing and flat memory (about 25 minutes) | isolated REAPER |
| `scripts/chaos/*.sh` | project switch mid-song, markers inserted live, 1000 regions, restarts | macOS, unlocked screen |

## Time budget

The checks on a pull request have a target of 5 minutes and a limit of 10 minutes. Anything that takes longer (the full soak) belongs in a nightly or manual run, not on every pull request.
