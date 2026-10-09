<script lang="ts">
  import type { Snippet } from "svelte";
  let { open, title, onclose, actions, children }: { open: boolean; title: string; onclose: () => void; actions?: Snippet; children: Snippet } = $props();
  let element = $state<HTMLDialogElement | null>(null);

  $effect(() => {
    if (!element) return;
    if (open && !element.open) element.showModal();
    if (!open && element.open) element.close();
  });
</script>

<dialog bind:this={element} aria-label={title} oncancel={(event) => { event.preventDefault(); onclose(); }}>
  <h2>{title}</h2>
  {@render children()}
  {#if actions}<footer>{@render actions()}</footer>{/if}
</dialog>

<style>
  dialog { min-width: min(420px, calc(100vw - 2 * var(--space-4))); padding: var(--space-5); border: 1px solid var(--line); border-radius: calc(var(--radius) * 1.6); background: var(--panel); color: var(--text); box-shadow: var(--shadow-raised); }
  dialog::backdrop { background: var(--backdrop); }
  h2 { margin: 0 0 var(--space-3) 0; font-size: var(--text-title); font-weight: 700; }
  footer { display: flex; justify-content: flex-end; gap: var(--space-3); margin-top: var(--space-4); }
</style>
