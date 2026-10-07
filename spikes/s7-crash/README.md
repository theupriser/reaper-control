# Spike S7: crash containment

Throwaway. Results: `docs/adr/ADR-009-crash-containment.md`.

```
source ~/.cargo/env
cd spikes/s7-crash && cargo build --release                      # panic = "unwind" (default)
cargo build --release --config 'profile.release.panic="abort"'   # control case
cp target/release/libs7_crash.dylib ../../.dev/reaper-test/UserPlugins/reaper_rc2s7-arm64.dylib   # remove other spike dylibs first
open -n -a /Applications/REAPER.app --args -cfgfile "$PWD/../../.dev/reaper-test/reaper.ini" "$PWD/../../testing/projects/s2/s2-clicks.rpp"
touch ../../.dev/reaper-test/RC2/s7-panic-main      # or s7-panic-thread
cat ../../.dev/reaper-test/RC2/spike-s7.log
```
Quit REAPER the way a user does (`NSRunningApplication.terminate()` on the instance's pid, same as Cmd+Q). SIGTERM is not a clean quit. A leftover `RC2/s7-running` file means the next start is safe mode; delete it to re-enable.
