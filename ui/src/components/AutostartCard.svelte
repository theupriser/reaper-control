<script lang="ts">
  import Panel from "../kit/Panel.svelte";
  import Switch from "../kit/Switch.svelte";
  import type { AutostartView } from "../lib/autostart";
  import { strings } from "../lib/strings";

  let { autostart, onToggle }: { autostart: AutostartView; onToggle: () => void } = $props();
</script>

<Panel title={strings.autostart.title} hint={strings.autostart.hint}>
  <Switch on={autostart.on} label={strings.autostart.switchLabel} disabled={!autostart.available} onchange={onToggle} />
  {#if autostart.problem}<p class="problem" role="status">{autostart.problem}</p>{/if}
</Panel>

<style>
  .problem { margin: 0; font-size: var(--text-small); color: var(--amber); }
</style>
