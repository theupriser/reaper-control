<script lang="ts">
  import Button from "../kit/Button.svelte";
  import type { SetlistInfo, SongInfo } from "../lib/generated/protocol";
  import { formatTime } from "../lib/performer";
  import { cardSummary } from "../lib/setlist-rows";
  import { strings } from "../lib/strings";

  let {
    setlist,
    songs,
    playing,
    onEdit,
    onPlay,
    onDelete,
  }: { setlist: SetlistInfo; songs: SongInfo[]; playing: boolean; onEdit: () => void; onPlay: () => void; onDelete: () => void } = $props();

  const summary = $derived(cardSummary(setlist, songs));
</script>

<section class="card" class:playing aria-label={setlist.name}>
  <div class="head">
    <div class="who">
      <div class="name">{setlist.name}</div>
      <div class="meta">{summary.count === 0 ? strings.setlists.songs(0) : strings.setlists.meta(summary.count, formatTime(summary.duration))}</div>
    </div>
    {#if playing}<span class="badge">{strings.setlists.playing}</span>{/if}
  </div>
  <p class="preview">{summary.count === 0 ? strings.setlists.previewEmpty : strings.setlists.preview(summary.preview, summary.more)}</p>
  {#if summary.missing > 0}<p class="warn" role="status">{strings.setlists.cardMissing(summary.missing)}</p>{/if}
  <div class="buttons">
    <Button onclick={onEdit}>{strings.setlists.edit}</Button>
    <Button disabled={playing} onclick={onPlay}>{playing ? strings.setlists.playing : strings.setlists.playThis}</Button>
    <Button kind="danger" label={strings.setlists.deleteSetlist(setlist.name)} onclick={onDelete}>✕</Button>
  </div>
</section>

<style>
  .card { display: flex; flex-direction: column; gap: 14px; padding: 20px; background: var(--panel); border: 1px solid var(--line); border-radius: 16px; }
  .playing { border-color: color-mix(in srgb, var(--green) 55%, var(--line)); }
  .head { display: flex; align-items: flex-start; justify-content: space-between; gap: var(--space-3); }
  .who { min-width: 0; }
  .name { font-size: 20px; font-weight: 800; line-height: 1.25; }
  .meta { margin-top: 4px; font-size: var(--text-small); color: var(--muted); }
  .badge { flex-shrink: 0; padding: 4px 10px; border-radius: 999px; background: color-mix(in srgb, var(--green) 14%, transparent); color: var(--green); font-size: 12px; font-weight: 800; letter-spacing: 0.04em; text-transform: uppercase; }
  .preview { margin: 0; min-height: 42px; color: var(--text-soft); font-size: var(--text-small); line-height: 1.5; }
  .warn { margin: 0; color: var(--amber); font-size: 13px; font-weight: 600; }
  .buttons { display: flex; gap: var(--space-2); margin-top: auto; }
  .buttons :global(button) { white-space: nowrap; }
  .buttons :global(button:first-child) { flex: 1; }
</style>
