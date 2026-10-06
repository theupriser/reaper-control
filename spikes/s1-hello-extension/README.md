# Spike S1: hello extension

Throwaway proof that a `reaper-rs` extension loads into REAPER and reads project data. Results: `docs/adr/ADR-002-extension-owned-playback.md`.

```
source ~/.cargo/env
cd spikes/s1-hello-extension
cargo build --release
# use a SEPARATE test config dir so your real REAPER setup is untouched
mkdir -p ../../.dev/reaper-test/UserPlugins
cp target/release/libs1_hello_extension.dylib ../../.dev/reaper-test/UserPlugins/reaper_rc2s1-arm64.dylib
open -n -a /Applications/REAPER.app --args -cfgfile "$PWD/../../.dev/reaper-test/reaper.ini" "$PWD/../../testing/projects/s1-sample.rpp"
# output: REAPER's ReaScript console window and ../../.dev/reaper-test/RC2/spike-s1.log
```

The crate is intentionally outside the main Cargo workspace (own `[workspace]` table), so it cannot affect CI or the lint wall. `.dev/` is git-ignored.
