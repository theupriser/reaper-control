<script lang="ts">
  import Switch from "../kit/Switch.svelte";
  import type { AutostartView } from "../lib/autostart";
  import { strings } from "../lib/strings";

  let { autostart, onToggle }: { autostart: AutostartView; onToggle: () => void } = $props();
</script>

<div class="card" class:on={autostart.on}>
  <Switch on={autostart.on} label={strings.autostart.switchLabel} disabled={!autostart.available} onchange={onToggle} />
  {#if autostart.problem}<div class="problem" role="status">{autostart.problem}</div>{/if}
</div>

<style>
  .card { display: flex; flex-direction: column; gap: 2px; margin-bottom: 8px; padding: 6px 14px; box-sizing: border-box; background: var(--panel); border: 1px solid var(--line); border-radius: 12px; }
  .on { border-color: color-mix(in srgb, var(--green) 55%, var(--line)); }
  .problem { padding-bottom: 6px; font-size: 12px; color: var(--amber); }
  .card :global(.switch) { font-size: 14px; font-weight: 700; min-height: 44px; text-align: left; }
</style>
