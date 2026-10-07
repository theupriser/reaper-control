<script lang="ts">
  import type { Command } from "../lib/types";

  let { onCommand }: { onCommand: (command: Command) => void } = $props();
  const buttons: Command[] = ["Play", "Pause", "Stop"];
</script>

<div class="bar">
  {#each buttons as command (command)}
    <button onclick={() => onCommand(command)}>{command}</button>
  {/each}
</div>

<style>
  .bar {
    display: flex;
    gap: 12px;
  }
  button {
    min-height: 56px;
    min-width: 120px;
    border-radius: 14px;
    border: 1px solid var(--line);
    background: var(--panel);
    color: var(--text);
    font-size: 18px;
    font-weight: 800;
    cursor: pointer;
  }
</style>
