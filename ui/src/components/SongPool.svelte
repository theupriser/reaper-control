<script lang="ts">
  import type { SongInfo } from "../lib/generated/protocol";
  import { matchingSongs } from "../lib/setlist-rows";
  import { strings } from "../lib/strings";

  let { songs, onAdd }: { songs: SongInfo[]; onAdd: (songId: string) => void } = $props();

  let query = $state("");
  const shown = $derived(matchingSongs(songs, query));
</script>

<section class="card" aria-label={strings.setlists.projectSongs}>
  <div class="top">
    <h2>{strings.setlists.projectSongs}</h2>
    <input type="search" placeholder={strings.setlists.search} aria-label={strings.setlists.search} bind:value={query} />
  </div>
  <div class="items" role="list">
    {#each shown as song (song.id)}
      <div class="item" role="listitem">
        <span class="name">{song.name}</span>
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
  .card { display: flex; flex-direction: column; min-height: 0; background: var(--panel); border: 1px solid var(--line); border-radius: 16px; overflow: hidden; }
  .top { padding: var(--space-4); border-bottom: 1px solid var(--line); }
  h2 { margin: 0; font-size: 16px; font-weight: 700; }
  input { box-sizing: border-box; width: 100%; margin-top: 10px; height: 40px; padding: 0 var(--space-3); border: 1px solid var(--line); border-radius: var(--radius); background: var(--bg); color: var(--text); font: inherit; font-size: var(--text-body); }
  .items { display: flex; flex-direction: column; gap: 2px; padding: var(--space-2); overflow-y: auto; }
  .item { display: flex; align-items: center; gap: var(--space-2); min-height: 46px; padding: 0 6px 0 10px; border-radius: var(--space-2); }
  .name { flex: 1; min-width: 0; font-size: var(--text-body); color: var(--text-soft); }
  button { display: flex; align-items: center; justify-content: center; width: 36px; height: 36px; border: 1px solid var(--line); border-radius: var(--space-2); background: transparent; color: var(--green); cursor: pointer; }
  button:hover { background: var(--tint-hover); }
  .hint { margin: 0; padding: var(--space-3); color: var(--muted); }
</style>
