# Spike S2: hand-over timing

Throwaway. Results: `docs/adr/ADR-005-handover-strategy.md`.

```
source ~/.cargo/env
cd spikes/s2-timing && cargo build --release
cp target/release/libs2_timing.dylib ../../.dev/reaper-test/UserPlugins/reaper_rc2s2-arm64.dylib
open -n -a /Applications/REAPER.app --args -cfgfile "$PWD/../../.dev/reaper-test/reaper.ini" "$PWD/../../testing/projects/s2/s2-clicks.rpp"
./run_matrix.sh ../../.dev/reaper-test/RC2 3     # 2 methods x 4 leads x 3 runs
python3 -I analyze.py ../../.dev/reaper-test/RC2/spike-s2.log
```

Single run: `echo "seek 15 [start_s]" > .dev/reaper-test/RC2/s2-run.txt` (or `stopseek`). The click project and its generator are in `testing/projects/s2/`.
