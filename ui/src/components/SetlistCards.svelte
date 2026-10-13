<script lang="ts">
  import type { SetlistInfo, SongInfo } from "../lib/generated/protocol";
  import { strings } from "../lib/strings";
  import SetlistCard from "./SetlistCard.svelte";

  let {
    setlists,
    songs,
    playing,
    onOpen,
    onPlay,
    onDelete,
    onCreate,
  }: {
    setlists: SetlistInfo[];
    songs: SongInfo[];
    playing: string | null;
    onOpen: (id: string) => void;
    onPlay: (id: string) => void;
    onDelete: (id: string) => void;
    onCreate: () => void;
  } = $props();
</script>

<div class="grid" role="list" aria-label={strings.setlists.listLabel}>
  {#each setlists as setlist (setlist.id)}
    <div role="listitem" class="cell">
      <SetlistCard {setlist} {songs} playing={playing === setlist.id} onEdit={() => onOpen(setlist.id)} onPlay={() => onPlay(setlist.id)} onDelete={() => onDelete(setlist.id)} />
    </div>
  {/each}
  <button class="new" onclick={onCreate}>
    <svg viewBox="0 0 24 24" width="28" height="28" fill="none" stroke="var(--green)" stroke-width="2.2" stroke-linecap="round"><path d="M12 5v14M5 12h14" /></svg>
    <span class="title">{strings.setlists.create}</span>
    <span class="hint">{strings.setlists.createHint}</span>
  </button>
</div>

<style>
  .grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(280px, 1fr)); gap: var(--space-4); align-content: start; }
  .cell { display: contents; }
  .cell :global(.card) { height: 100%; box-sizing: border-box; }
  .new { display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 10px; min-height: 196px; border: 2px dashed var(--line); border-radius: 16px; background: transparent; color: var(--muted); font: inherit; cursor: pointer; }
  .new:hover { background: var(--tint-hover); }
  .title { font-size: 15px; font-weight: 700; }
  .hint { font-size: 13px; }
</style>
