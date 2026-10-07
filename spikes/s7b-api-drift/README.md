# Spike S7b: API drift

Asks REAPER for every function the extension would use (`GetFunc`) and logs the REAPER version and what is missing, to `<resource>/RC2/spike-s7b.log`. Throwaway code.

```
source ~/.cargo/env
cd spikes/s7b-api-drift && cargo build --release
cp target/release/libs7b_api_drift.dylib <dir>/UserPlugins/reaper_s7b-arm64.dylib
open -n -a /Applications/REAPER.app --args -cfgfile <dir>/reaper.ini <project>
```
