<script lang="ts">
  import Panel from "../kit/Panel.svelte";

  let { title, intro, steps, example }: { title: string; intro?: string; steps: readonly string[]; example?: string } = $props();
</script>

<Panel {title}>
  {#if intro}<p class="intro">{intro}</p>{/if}
  <ul>
    {#each steps as step}<li>{step}</li>{/each}
  </ul>
  {#if example}<code>{example}</code>{/if}
</Panel>

<style>
  .intro { margin: 0; color: var(--muted); }
  ul { margin: 0; padding-left: 20px; display: flex; flex-direction: column; gap: 8px; }
  code { align-self: flex-start; padding: 4px 10px; border-radius: var(--radius); background: var(--line); font-family: ui-monospace, monospace; }
</style>
