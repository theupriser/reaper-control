<script lang="ts">
  import type { Snippet } from "svelte";
  let {
    selected = false,
    onclick,
    leading,
    trailing,
    children,
  }: { selected?: boolean; onclick?: () => void; leading?: Snippet; trailing?: Snippet; children: Snippet } = $props();
</script>

<svelte:element this={onclick ? "button" : "div"} role={onclick ? undefined : "listitem"} class="row" class:selected class:clickable={!!onclick} {onclick}>
  {#if leading}<span class="side">{@render leading()}</span>{/if}
  <span class="body">{@render children()}</span>
  {#if trailing}<span class="side">{@render trailing()}</span>{/if}
</svelte:element>

<style>
  .row { display: flex; align-items: center; gap: var(--space-3); min-height: var(--control-height); padding: 0 var(--control-padding); border: none; border-bottom: 1px solid var(--line-soft); background: transparent; color: var(--text); font: inherit; font-size: var(--text-body); text-align: left; width: 100%; }
  .clickable { cursor: pointer; }
  .clickable:hover { background: var(--tint-hover); }
  .selected { background: var(--tint-ok); box-shadow: inset 3px 0 0 var(--green); }
  .body { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .side { display: flex; align-items: center; gap: var(--space-2); flex-shrink: 0; color: var(--muted); }
</style>
