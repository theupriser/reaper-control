<script lang="ts">
  import Button from "../kit/Button.svelte";
  import type { SetlistInfo } from "../lib/generated/protocol";
  import { strings } from "../lib/strings";

  let {
    setlists,
    selected,
    playing,
    onOpen,
    onCreate,
  }: { setlists: SetlistInfo[]; selected: string | null; playing: string | null; onOpen: (id: string) => void; onCreate: () => void } = $props();
</script>

<section class="card" aria-label={strings.setlists.listLabel}>
  <div class="top"><Button kind="primary" onclick={onCreate}>{strings.setlists.create}</Button></div>
  <ul class="items">
    {#each setlists as setlist (setlist.id)}
      <li>
        <button class="item" class:selected={selected === setlist.id} onclick={() => onOpen(setlist.id)}>
          <span class="name">{setlist.name}</span>
          <span class="meta">{strings.setlists.listMeta(setlist.entries.length, playing === setlist.id)}</span>
        </button>
      </li>
    {:else}
      <li class="hint">{strings.setlists.none}</li>
    {/each}
  </ul>
</section>

<style>
  .card { display: flex; flex-direction: column; background: var(--panel); border: 1px solid var(--line); border-radius: 16px; overflow: hidden; }
  .top { padding: var(--space-4); border-bottom: 1px solid var(--line); display: flex; flex-direction: column; }
  .items { list-style: none; margin: 0; display: flex; flex-direction: column; gap: var(--space-1); padding: var(--space-2); overflow-y: auto; }
  .item { width: 100%; display: flex; flex-direction: column; gap: 2px; padding: 10px var(--space-3); border: none; border-radius: var(--radius); background: transparent; color: var(--text); font: inherit; text-align: left; cursor: pointer; }
  .item:hover { background: var(--tint-hover); }
  .name { font-size: var(--text-body); font-weight: 700; }
  .meta { font-size: var(--text-small); color: var(--muted); }
  .selected { background: var(--raised); }
  .selected .name { color: var(--green); }
  .hint { padding: var(--space-3); color: var(--muted); }
</style>
