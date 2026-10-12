# Testing

Everything the tests read or run that is not Rust or UI code.

```
testing/
  README.md
  data/
    scenarios/   performance scenarios (JSON), read by the reaper-port tests
    vectors/     protocol and marker test vectors (JSON), read by the protocol and catalogue tests
  scripts/       scripts that run against a real REAPER
    chaos/       project switches, live markers and a 1000-region project
```

- `data/` holds fixtures that tests read. No scripts and no REAPER projects.
- `scripts/` holds scripts that start an isolated REAPER (a throwaway copy under the git-ignored `.dev/`, never your own REAPER). Each script starts with a header that says what it checks and what it needs. Run them from the repository root.
- `projects/` is a local, git-ignored folder with sample REAPER projects for trying things by hand.

An architecture test fails when something else sits in `testing/`.

## Scripts

| Script | What it checks | Needs |
|---|---|---|
| `scripts/smoke_extension.sh` | the packaged extension loads and listens | macOS, packaged extension |
| `scripts/security_check.sh` | where the extension listens, who can read the token, a wrong token, junk input | a running isolated REAPER |
| `scripts/resource_audit.sh` | idle CPU and memory of the app, REAPER and the extension | macOS, release build |
| `scripts/soak.sh` | 40 songs in a row: hand-over timing and flat memory (about 25 minutes) | isolated REAPER |
| `scripts/chaos/*.sh` | project switch mid-song, markers inserted live, 1000 regions, restarts | macOS, unlocked screen |

## Time budget

The checks on a pull request have a target of 5 minutes and a limit of 10 minutes. Anything that takes longer (the full soak) belongs in a nightly or manual run, not on every pull request.
