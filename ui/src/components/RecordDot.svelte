<script lang="ts">
  import { strings } from "../lib/strings";
  let { armed, onToggle, disabled = false }: { armed: boolean; onToggle: () => void; disabled?: boolean } = $props();
</script>

<button class="record" class:armed aria-label={strings.performer.toggleRecording} aria-pressed={armed} {disabled} onclick={onToggle}>
  <span class="dot"></span>
</button>

<style>
  .record {
    display: flex;
    align-items: center;
    padding: 0;
    margin-right: 8px;
    border: none;
    background: none;
    cursor: pointer;
  }
  .record:disabled { cursor: default; opacity: 0.5; }
  .dot {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background: var(--on-strong);
  }
  .armed .dot { background: red; }
</style>
