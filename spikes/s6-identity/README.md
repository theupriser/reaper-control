# Spike S6: identity

Throwaway. Reads region/marker GUIDs and runs edits from a command file. Results: `docs/adr/ADR-008-song-identity.md`.

```
source ~/.cargo/env
cd spikes/s6-identity && cargo build --release
cp target/release/libs6_identity.dylib ../../.dev/reaper-test/UserPlugins/reaper_rc2s6-arm64.dylib
open -n -a /Applications/REAPER.app --args -cfgfile "$PWD/../../.dev/reaper-test/reaper.ini" <copy of a project>
echo "dump label" > ../../.dev/reaper-test/RC2/s6-cmd     # see src/lib.rs for all commands
# output: ../../.dev/reaper-test/RC2/spike-s6.log
```
