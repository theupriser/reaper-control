<script lang="ts">
  import Button from "../kit/Button.svelte";
  import { type DragHandlers, type DragPoint, insertionIndex } from "../lib/drag-drop";
  import type { SongInfo } from "../lib/generated/protocol";
  import { formatTime } from "../lib/performer";
  import type { SetlistDraft } from "../lib/setlist-edit";
  import { type EntryRow, entryRows, totalDuration } from "../lib/setlist-rows";
  import { strings } from "../lib/strings";
  import SetlistEntries from "./SetlistEntries.svelte";
  import SongPool from "./SongPool.svelte";

  let {
    draft,
    songs,
    changed,
    problem,
    stale,
    saved,
    playing,
    onBack,
    onRename,
    onMoveTo,
    onAdd,
    onRemove,
    onSave,
    onDiscard,
    onReload,
    onPlay,
  }: {
    draft: SetlistDraft;
    songs: SongInfo[];
    changed: boolean;
    problem: string | null;
    stale: boolean;
    saved: boolean;
    playing: boolean;
    onBack: () => void;
    onRename: (name: string) => void;
    onMoveTo: (entryId: number, index: number) => void;
    onAdd: (songId: string, index?: number) => void;
    onRemove: (entryId: number) => void;
    onSave: () => void;
    onDiscard: () => void;
    onReload: () => void;
    onPlay: () => void;
  } = $props();

  type Lifted = { kind: "entry"; id: number; name: string } | { kind: "song"; id: string; name: string };

  const rows = $derived(entryRows(draft, songs));
  const missing = $derived(rows.filter((row) => row.missing).length);
  const used = $derived(new Set(draft.entries.map((entry) => entry.song_id)));

  let list = $state<HTMLElement>();
  let lifted = $state<Lifted | null>(null);
  let pointer = $state<DragPoint | null>(null);

  const middles = () => [...(list?.querySelectorAll<HTMLElement>("[data-row]") ?? [])].map((row) => row.getBoundingClientRect()).map((box) => box.top + box.height / 2);
  const over = (point: DragPoint) => {
    const box = list?.getBoundingClientRect();
    return box !== undefined && point.x >= box.left && point.x <= box.right && point.y >= box.top && point.y <= box.bottom;
  };
  // A song only lands when it is let go over the list; an entry can be moved from anywhere on the screen.
  const landing = (point: DragPoint | null): number | null =>
    lifted && point && (lifted.kind === "entry" || over(point)) ? insertionIndex(middles(), point.y) : null;
  const insertAt = $derived(landing(pointer));

  const handlers = (item: Lifted): DragHandlers => ({
    onStart: () => (lifted = item),
    onMove: (point) => (pointer = point),
    onDrop: (point) => {
      const at = landing(point);
      lifted = null;
      pointer = null;
      if (at === null) return;
      if (item.kind === "entry") onMoveTo(item.id, at);
      else onAdd(item.id, at);
    },
    onCancel: () => {
      lifted = null;
      pointer = null;
    },
  });
  const entryDrag = (row: EntryRow) => handlers({ kind: "entry", id: row.id, name: row.name });
  const songDrag = (song: SongInfo) => handlers({ kind: "song", id: song.id, name: song.name });
</script>

<div class="screen" class:dragging={lifted !== null}>
  <button class="back" onclick={onBack}>
    <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"><path d="M15 6l-6 6 6 6" /></svg>
    {strings.setlists.back}
  </button>

  <div class="head">
    <div class="who">
      <input aria-label={strings.setlists.name} value={draft.name} oninput={(event) => onRename(event.currentTarget.value)} />
      <div class="meta">
        {strings.setlists.meta(rows.length, formatTime(totalDuration(rows)))}
        {#if problem}<span class="warn" role="status"> · {strings.setlists.nameNeeded}</span>{:else if changed}<span class="warn" role="status"> · {strings.setlists.unsaved}</span>{/if}
      </div>
    </div>
    <div class="buttons">
      {#if saved}<Button disabled={playing || changed} onclick={onPlay}>{strings.setlists.play}</Button>{/if}
      <Button disabled={!changed} onclick={onDiscard}>{strings.setlists.discard}</Button>
      <Button kind="primary" disabled={!changed || problem !== null} onclick={onSave}>{strings.setlists.save}</Button>
    </div>
  </div>

  {#if stale}
    <p class="banner" role="status">{strings.setlists.changedInReaper} <Button onclick={onReload}>{strings.setlists.reload}</Button></p>
  {/if}
  {#if missing > 0}<p class="banner" role="status">{strings.setlists.missingBanner(missing)}</p>{/if}

  <div class="columns">
    <SetlistEntries {rows} liftedEntry={lifted?.kind === "entry" ? lifted.id : null} {insertAt} bind:element={list} drag={entryDrag} {onRemove} />
    <SongPool {songs} {used} lifted={lifted?.kind === "song" ? lifted.id : null} drag={songDrag} onAdd={(songId) => onAdd(songId)} />
  </div>
</div>

{#if lifted && pointer}<div class="ghost" style:left="{pointer.x + 14}px" style:top="{pointer.y + 8}px">{lifted.name}</div>{/if}

<style>
  .screen { display: flex; flex-direction: column; gap: 14px; padding: 24px 32px 28px; min-height: 0; height: 100%; box-sizing: border-box; }
  .dragging { user-select: none; cursor: grabbing; }
  .back { display: inline-flex; align-items: center; gap: 6px; align-self: flex-start; height: 32px; padding: 0; border: none; background: transparent; color: var(--muted); font: inherit; font-size: 14px; font-weight: 600; cursor: pointer; }
  .back:hover { color: var(--text); }
  .head { display: flex; flex-wrap: wrap; align-items: center; justify-content: space-between; gap: var(--space-4); }
  .who { flex: 1 1 16rem; min-width: 0; }
  input { box-sizing: border-box; width: 100%; max-width: 460px; padding: 0 0 2px; border: none; border-bottom: 1px solid var(--line); background: transparent; color: var(--text); font: inherit; font-size: 28px; font-weight: 800; }
  .meta { margin-top: 4px; font-size: var(--text-small); color: var(--muted); }
  .warn { color: var(--amber); font-weight: 600; }
  .buttons { display: flex; gap: var(--space-2); flex-shrink: 0; }
  .banner { display: flex; align-items: center; gap: var(--space-3); margin: 0; padding: 9px 14px; border-radius: var(--radius); background: color-mix(in srgb, var(--amber) 12%, transparent); color: var(--amber); font-size: 14px; }
  .columns { flex: 1; min-height: 0; display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: var(--space-4); }
  .ghost { position: fixed; z-index: 10; max-width: 260px; padding: 8px 14px; border: 1px solid var(--green); border-radius: 10px; background: var(--raised); box-shadow: var(--shadow-raised); font-weight: 600; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; pointer-events: none; }
</style>
