<script lang="ts">
  import Switch from "../kit/Switch.svelte";
  import { strings } from "../lib/strings";
  let {
    recordArmed,
    autoResume,
    countInOnMarker,
    onToggleRecord,
    onToggleAutoResume,
    onToggleCountIn,
  }: {
    recordArmed: boolean;
    autoResume: boolean;
    countInOnMarker: boolean;
    onToggleRecord: () => void;
    onToggleAutoResume: () => void;
    onToggleCountIn: () => void;
  } = $props();
</script>

<div class="options" role="group" aria-label={strings.player.optionsLabel}>
  <span class="heading">{strings.player.options}</span>
  <Switch on={recordArmed} label={strings.player.armRecording} onchange={onToggleRecord} />
  <Switch on={autoResume} label={strings.player.autoResume} onchange={onToggleAutoResume} />
  <Switch on={countInOnMarker} label={strings.player.countInOnMarker} onchange={onToggleCountIn} />
</div>

<style>
  .options { display: flex; flex-wrap: wrap; align-items: center; justify-content: center; gap: var(--space-2) 28px; padding-top: var(--space-3); border-top: 1px solid var(--line); }
  .heading { font-size: 12px; font-weight: 700; letter-spacing: 1px; color: var(--muted); }
</style>
