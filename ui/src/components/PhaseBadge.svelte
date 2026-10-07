<script lang="ts">
  import type { Phase } from "../lib/types";

  let { phase }: { phase: Phase } = $props();
</script>

<span class="badge {phase.toLowerCase()}" data-testid="phase">{phase}</span>

<style>
  .badge {
    padding: 6px 14px;
    border-radius: 999px;
    font-weight: 800;
    font-size: 14px;
    background: var(--line);
  }
  .playing {
    background: rgba(76, 175, 80, 0.2);
    color: var(--green);
  }
  .paused {
    background: rgba(255, 193, 7, 0.2);
    color: var(--amber);
  }
</style>
