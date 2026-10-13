<script lang="ts">
  import { dragHandle, type DragHandlers } from "../lib/drag-drop";

  let { label, handlers }: { label: string; handlers: DragHandlers } = $props();
</script>

<span class="grip" role="img" aria-label={label} use:dragHandle={handlers}>
  <svg viewBox="0 0 24 24" width="20" height="20" fill="currentColor"><path d="M9 5h2v2H9zm4 0h2v2h-2zM9 11h2v2H9zm4 0h2v2h-2zM9 17h2v2H9zm4 0h2v2h-2z" /></svg>
</span>

<style>
  .grip { display: flex; align-items: center; justify-content: center; width: 28px; height: 36px; flex-shrink: 0; color: var(--muted); cursor: grab; touch-action: none; -webkit-user-select: none; user-select: none; }
  .grip:active { cursor: grabbing; }
</style>
