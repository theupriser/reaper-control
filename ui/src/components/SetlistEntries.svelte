<script lang="ts">
  import type { DragHandlers } from "../lib/drag-drop";
  import type { EntryRow } from "../lib/setlist-rows";
  import { strings } from "../lib/strings";
  import SetlistEntryRow from "./SetlistEntryRow.svelte";

  let {
    rows,
    liftedEntry,
    insertAt,
    element = $bindable(),
    drag,
    onMove,
    onRemove,
  }: {
    rows: EntryRow[];
    liftedEntry: number | null;
    insertAt: number | null;
    element: HTMLElement | undefined;
    drag: (row: EntryRow) => DragHandlers;
    onMove: (entryId: number, direction: -1 | 1) => void;
    onRemove: (entryId: number) => void;
  } = $props();
</script>

<section class="card" aria-label={strings.setlists.editorLabel}>
  <div class="top"><h2>{strings.setlists.inThisSetlist}</h2><span>{strings.setlists.dragToReorder}</span></div>
  <div class="entries" role="list" aria-label={strings.setlists.entriesLabel} bind:this={element}>
    {#each rows as row, at (row.id)}
      <SetlistEntryRow
        {row}
        first={at === 0}
        last={at === rows.length - 1}
        lifted={liftedEntry === row.id}
        dropBefore={insertAt === at}
        drag={drag(row)}
        onUp={() => onMove(row.id, -1)}
        onDown={() => onMove(row.id, 1)}
        onRemove={() => onRemove(row.id)} />
    {/each}
    <div class="end" class:active={insertAt === rows.length}>{rows.length === 0 ? strings.setlists.empty : strings.setlists.dropHint}</div>
  </div>
</section>

<style>
  .card { display: flex; flex-direction: column; min-width: 0; min-height: 0; background: var(--panel); border: 1px solid var(--line); border-radius: 16px; overflow: hidden; }
  .top { display: flex; align-items: baseline; justify-content: space-between; gap: var(--space-3); padding: 14px 18px; border-bottom: 1px solid var(--line); }
  .top span { font-size: 13px; color: var(--muted); }
  h2 { margin: 0; font-size: 17px; font-weight: 800; }
  .entries { display: flex; flex-direction: column; gap: 3px; padding: 8px 10px; overflow-y: auto; min-height: 0; }
  .end { flex-shrink: 0; margin: 6px 4px 0; min-height: 52px; display: flex; align-items: center; justify-content: center; border: 2px dashed var(--line); border-radius: 10px; color: var(--muted); font-size: 14px; font-weight: 600; text-align: center; }
  .end.active { border-color: var(--green); color: var(--green); }
</style>
