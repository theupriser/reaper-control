<script lang="ts">
  import { strings } from "../lib/strings";
  import { usageLevel, type SystemStatsView } from "../lib/performer";

  let { stats }: { stats: SystemStatsView } = $props();
</script>

<div class="stats" aria-label={strings.performer.systemStatus}>
  <span class="midi" class:active={stats.midiActive} title={strings.performer.midiActivity}>
    <svg viewBox="0 0 24 24" width="18" height="18">
      <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 1c4.97 0 9 4.03 9 9s-4.03 9-9 9-9-4.03-9-9 4.03-9 9-9z" />
      <circle cx="7" cy="8" r="1" /><circle cx="12" cy="6" r="1" /><circle cx="17" cy="8" r="1" />
      <circle cx="9.5" cy="10" r="1" /><circle cx="14.5" cy="10" r="1" />
    </svg>
  </span>
  <span class="dot" class:connected={stats.connected} title={stats.connected ? strings.connection.connected : strings.connection.disconnected}></span>
  <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
    <path d="M18 20V10" /><path d="M12 20V4" /><path d="M6 20v-6" />
  </svg>
  <span class="usage" title={strings.performer.cpu(stats.cpu)}>
    <span class="bar {usageLevel(stats.cpu)}" style="width: {stats.cpu}%"></span>
  </span>
</div>

<style>
  .stats {
    display: flex;
    align-items: center;
    gap: 8px;
    color: var(--muted);
  }
  .midi svg { fill: var(--muted); transition: fill 0.1s ease, filter 0.1s ease; }
  .midi.active svg { fill: var(--amber); filter: drop-shadow(0 0 2px var(--amber)); }
  .dot {
    width: 10px;
    height: 10px;
    margin-left: 4px;
    border-radius: 50%;
    background: var(--danger);
    box-shadow: 0 0 5px var(--danger);
  }
  .dot.connected { background: var(--green); box-shadow: 0 0 5px var(--green); }
  .usage {
    width: 40px;
    height: 10px;
    background: var(--track);
    border-radius: 4px;
    overflow: hidden;
  }
  .bar { display: block; height: 100%; transition: width 0.5s ease; }
  .low { background: var(--green); }
  .medium { background: var(--amber); }
  .high { background: var(--danger); }
</style>
