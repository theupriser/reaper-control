<script lang="ts">
  import DragGrip from "./DragGrip.svelte";
  import type { DragHandlers } from "../lib/drag-drop";
  import type { SongInfo } from "../lib/generated/protocol";
  import { formatTime } from "../lib/performer";
  import { songLength } from "../lib/performer-view";
  import { matchingSongs } from "../lib/setlist-rows";
  import { strings } from "../lib/strings";

  let {
    songs,
    used,
    lifted,
    drag,
    onAdd,
  }: { songs: SongInfo[]; used: Set<string>; lifted: string | null; drag: (song: SongInfo) => DragHandlers; onAdd: (songId: string) => void } = $props();

  let query = $state("");
  const shown = $derived(matchingSongs(songs, query));
</script>

<section class="card" aria-label={strings.setlists.projectSongs}>
  <div class="top">
    <div class="title"><h2>{strings.setlists.projectSongs}</h2><span>{strings.setlists.poolHint}</span></div>
    <input type="search" placeholder={strings.setlists.search} aria-label={strings.setlists.search} bind:value={query} />
  </div>
  <div class="items" role="list">
    {#each shown as song (song.id)}
      <div class="item" class:lifted={lifted === song.id} role="listitem">
        <DragGrip label={strings.setlists.dragSong(song.name)} handlers={drag(song)} />
        <span class="name">{song.name}</span>
        {#if used.has(song.id)}<span class="used">{strings.setlists.inSetlist}</span>{/if}
        <span class="time">{formatTime(songLength(song))}</span>
        <button aria-label={strings.setlists.addSong(song.name)} onclick={() => onAdd(song.id)}>
          <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round"><path d="M12 5v14M5 12h14" /></svg>
        </button>
      </div>
    {:else}
      <p class="hint">{strings.setlists.noSongsFound}</p>
    {/each}
  </div>
</section>

<style>
  .card { display: flex; flex-direction: column; min-width: 0; min-height: 0; background: var(--panel); border: 1px solid var(--line); border-radius: 16px; overflow: hidden; }
  .top { padding: 14px 18px; border-bottom: 1px solid var(--line); }
  .title { display: flex; align-items: baseline; justify-content: space-between; gap: var(--space-3); }
  .title span { font-size: 13px; color: var(--muted); }
  h2 { margin: 0; font-size: 17px; font-weight: 800; }
  input { box-sizing: border-box; width: 100%; margin-top: 10px; height: 40px; padding: 0 var(--space-3); border: 1px solid var(--line); border-radius: var(--radius); background: var(--bg); color: var(--text); font: inherit; font-size: var(--text-body); }
  .items { display: flex; flex-direction: column; gap: 3px; padding: 8px 10px; overflow-y: auto; min-height: 0; }
  .item { display: flex; align-items: center; gap: 10px; min-height: 50px; padding: 0 6px 0 4px; border-radius: 10px; flex-shrink: 0; }
  .lifted { opacity: 0.45; }
  .name { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: 16px; color: var(--text-soft); }
  .used { font-size: 12px; font-weight: 700; color: var(--green); }
  .time { min-width: 48px; text-align: right; font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace; font-size: 14px; color: var(--muted); }
  button { display: flex; align-items: center; justify-content: center; width: 36px; height: 36px; border: 1px solid var(--line); border-radius: 8px; background: transparent; color: var(--green); cursor: pointer; }
  button:hover { background: var(--tint-hover); }
  .hint { margin: 0; padding: var(--space-3); color: var(--muted); }
</style>
