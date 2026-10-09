<script lang="ts">
  import { strings } from "../lib/strings";
  let { armed, onToggle }: { armed: boolean; onToggle: () => void } = $props();
</script>

<button class="record" class:armed aria-label={strings.performer.toggleRecording} aria-pressed={armed} onclick={onToggle}>
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
  .dot {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background: var(--on-strong);
  }
  .armed .dot { background: red; }
</style>
