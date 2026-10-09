<script lang="ts">
  import { strings } from "../lib/strings";
  import Toggle from "../kit/Toggle.svelte";
  let {
    autoResume,
    countInOnMarker,
    onAutoResume,
    onCountIn,
    disabled = false,
  }: { autoResume: boolean; countInOnMarker: boolean; onAutoResume: () => void; onCountIn: () => void; disabled?: boolean } = $props();
</script>

<div class="toggles">
  <Toggle on={autoResume} label={strings.performer.autoResume(autoResume)} onchange={onAutoResume} {disabled} />
  <Toggle on={countInOnMarker} label={strings.performer.countInOnMarker(countInOnMarker)} onchange={onCountIn} {disabled} />
</div>

<style>
  .toggles {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1rem;
    width: 100%;
    max-width: 500px;
    margin: 0 auto;
  }
</style>
