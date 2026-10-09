<script lang="ts">
  import type { Snippet } from "svelte";
  let { title, hint, action, children }: { title?: string; hint?: string; action?: Snippet; children: Snippet } = $props();
</script>

<section class="panel">
  {#if title || action}
    <header>
      <div>
        {#if title}<h2>{title}</h2>{/if}
        {#if hint}<p class="hint">{hint}</p>{/if}
      </div>
      {#if action}{@render action()}{/if}
    </header>
  {/if}
  {@render children()}
</section>

<style>
  .panel { background: var(--panel); border: 1px solid var(--line); border-radius: calc(var(--radius) * 1.6); padding: var(--space-4) var(--space-5); display: flex; flex-direction: column; gap: var(--space-3); }
  header { display: flex; justify-content: space-between; align-items: center; gap: var(--space-4); }
  h2 { margin: 0; font-size: var(--text-title); font-weight: 700; }
  .hint { margin: 0; font-size: var(--text-small); color: var(--muted); }
</style>
