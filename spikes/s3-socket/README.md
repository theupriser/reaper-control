# Spike S3: local link

Throwaway. Results: `docs/adr/ADR-003-local-link.md`.

```
source ~/.cargo/env
cd spikes/s3-socket && cargo build --release
cp target/release/libs3_socket.dylib ../../.dev/reaper-test/UserPlugins/reaper_rc2s3-arm64.dylib   # remove other spike dylibs first
open -n -a /Applications/REAPER.app --args -cfgfile "$PWD/../../.dev/reaper-test/reaper.ini" "$PWD/../../testing/projects/s2/s2-clicks.rpp"
C=target/release/s3-client; E=../../.dev/reaper-test/RC2/s3-endpoint.txt
$C $E watch 10; $C $E ping 10; $C $E badtoken; $C $E churn 200; $C $E slow 15
cat ../../.dev/reaper-test/RC2/spike-s3.log
```
