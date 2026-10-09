<script lang="ts">
  import Button from "../kit/Button.svelte";
  import type { SongInfo } from "../lib/generated/protocol";
  import { formatTime } from "../lib/performer";
  import type { SetlistDraft } from "../lib/setlist-edit";
  import { entryRows, totalDuration } from "../lib/setlist-rows";
  import { strings } from "../lib/strings";
  import SetlistEntryRow from "./SetlistEntryRow.svelte";

  let {
    draft,
    songs,
    changed,
    problem,
    stale,
    saved,
    playing,
    onRename,
    onMove,
    onRemove,
    onSave,
    onDiscard,
    onReload,
    onPlay,
    onDelete,
  }: {
    draft: SetlistDraft;
    songs: SongInfo[];
    changed: boolean;
    problem: string | null;
    stale: boolean;
    saved: boolean;
    playing: boolean;
    onRename: (name: string) => void;
    onMove: (entryId: number, direction: -1 | 1) => void;
    onRemove: (entryId: number) => void;
    onSave: () => void;
    onDiscard: () => void;
    onReload: () => void;
    onPlay: () => void;
    onDelete: () => void;
  } = $props();

  const rows = $derived(entryRows(draft, songs));
  const missing = $derived(rows.filter((row) => row.missing).length);
</script>

<section class="card" aria-label={strings.setlists.editorLabel}>
  <div class="head">
    <div class="who">
      <input aria-label={strings.setlists.name} value={draft.name} oninput={(event) => onRename(event.currentTarget.value)} />
      <div class="meta">{strings.setlists.meta(rows.length, formatTime(totalDuration(rows)))}</div>
    </div>
    {#if saved}
      <div class="buttons">
        <Button disabled={playing} onclick={onPlay}>{strings.setlists.play}</Button>
        <Button kind="danger" onclick={onDelete}>{strings.setlists.delete}</Button>
      </div>
    {/if}
  </div>

  {#if stale}
    <p class="banner" role="status">{strings.setlists.changedInReaper} <Button onclick={onReload}>{strings.setlists.reload}</Button></p>
  {/if}
  {#if missing > 0}<p class="banner" role="status">{strings.setlists.missingBanner(missing)}</p>{/if}

  <div class="entries" role="list" aria-label={strings.setlists.entriesLabel}>
    {#each rows as row, at (row.id)}
      <SetlistEntryRow {row} first={at === 0} last={at === rows.length - 1} onUp={() => onMove(row.id, -1)} onDown={() => onMove(row.id, 1)} onRemove={() => onRemove(row.id)} />
    {:else}
      <p class="hint">{strings.setlists.empty}</p>
    {/each}
  </div>

  <div class="actions">
    <Button kind="primary" disabled={!changed || problem !== null} onclick={onSave}>{strings.setlists.save}</Button>
    <Button disabled={!changed} onclick={onDiscard}>{strings.setlists.discard}</Button>
    {#if problem}<span class="message warn" role="status">{strings.setlists.nameNeeded}</span>{:else if changed}<span class="message" role="status">{strings.setlists.unsaved}</span>{/if}
  </div>
</section>

<style>
  .card { display: flex; flex-direction: column; min-width: 0; background: var(--panel); border: 1px solid var(--line); border-radius: 16px; overflow: hidden; }
  .head { display: flex; flex-wrap: wrap; align-items: center; justify-content: space-between; gap: var(--space-4); padding: var(--space-4) 20px; border-bottom: 1px solid var(--line); }
  .who { min-width: 12rem; flex: 1 1 12rem; }
  input { box-sizing: border-box; width: 100%; padding: 0; border: none; border-bottom: 1px solid transparent; background: transparent; color: var(--text); font: inherit; font-size: var(--text-title); font-weight: 800; }
  input:hover, input:focus { border-bottom-color: var(--line); }
  .meta { margin-top: 2px; font-size: 13px; color: var(--muted); }
  .buttons { display: flex; gap: var(--space-2); flex-shrink: 0; }
  .banner { display: flex; align-items: center; gap: var(--space-3); margin: 14px 20px 0; padding: 10px 14px; border-radius: var(--radius); background: color-mix(in srgb, var(--amber) 12%, transparent); color: var(--amber); font-size: 14px; }
  .entries { padding: 10px var(--space-3); overflow-y: auto; }
  .actions { display: flex; align-items: center; gap: var(--space-3); flex-wrap: wrap; padding: var(--space-3) 20px var(--space-4); border-top: 1px solid var(--line); }
  .message { font-size: var(--text-small); color: var(--muted); }
  .warn { color: var(--amber); }
  .hint { color: var(--muted); margin: var(--space-3); }
</style>
