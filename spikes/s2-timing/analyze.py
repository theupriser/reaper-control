"""Summarises spike-s2.log: per run, cut error (pos at trigger tick minus song end) and the time until
playback of the next song is seen moving, measured from the trigger tick."""
import re, statistics, sys
from collections import defaultdict

END_A, START_B = 10.0, 12.0
runs, meta = defaultdict(list), {}
for line in open(sys.argv[1]):
    m = re.match(r"BEGIN run=(\d+) method=(\w+) lead_ms=([\d.]+)", line)
    if m:
        meta[int(m[1])] = (m[2], float(m[3])); continue
    m = re.match(r"([TE]) run=(\d+) ms=([\d.]+).*?(?:pos_before|pos)=([\d.]+)", line)
    if m:
        runs[int(m[2])].append((m[1], float(m[3]), float(m[4])))
rows = defaultdict(list)
for rid, ev in runs.items():
    trig = next((e for e in ev if e[0] == "E"), None)
    if not trig: continue
    cut = (trig[2] - END_A) * 1000
    wall_b = next((ms - (pos - START_B) * 1000 for k, ms, pos in ev if k == "T" and ms > trig[1] and pos > START_B + 0.001), None)
    rows[meta[rid]].append((cut, None if wall_b is None else wall_b - trig[1]))
print(f"{'method':9} {'lead':>5} {'n':>2} {'cut ms (min/mean/max)':>26} {'next song moving after trigger, ms (min/mean/max)':>52}")
for (method, lead), v in sorted(rows.items()):
    cuts = [c for c, _ in v]; gaps = [g for _, g in v if g is not None]
    f = lambda xs: f"{min(xs):7.1f}/{statistics.mean(xs):7.1f}/{max(xs):7.1f}"
    print(f"{method:9} {lead:5.0f} {len(v):2} {f(cuts):>26} {f(gaps) if gaps else 'n/a':>52}")
