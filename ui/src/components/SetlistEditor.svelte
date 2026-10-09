<script lang="ts">
  import Button from "../kit/Button.svelte";
  import Panel from "../kit/Panel.svelte";
  import type { SongInfo } from "../lib/generated/protocol";
  import { missingEntries, type SetlistDraft } from "../lib/setlist-edit";
  import { strings } from "../lib/strings";
  import SetlistEntryRow from "./SetlistEntryRow.svelte";

  let {
    draft,
    songs,
    changed,
    problem,
    stale,
    onRename,
    onAdd,
    onMove,
    onRemove,
    onSave,
    onDiscard,
    onReload,
  }: {
    draft: SetlistDraft;
    songs: SongInfo[];
    changed: boolean;
    problem: string | null;
    stale: boolean;
    onRename: (name: string) => void;
    onAdd: (songId: string) => void;
    onMove: (entryId: number, direction: -1 | 1) => void;
    onRemove: (entryId: number) => void;
    onSave: () => void;
    onDiscard: () => void;
    onReload: () => void;
  } = $props();

  let chosen = $state("");
  const missing = $derived(missingEntries(draft, songs));
  const nameOf = (songId: string) => songs.find((song) => song.id === songId)?.name ?? songId;
</script>

<Panel title={draft.name || strings.setlists.newName}>
  <label class="field">{strings.setlists.name}
    <input value={draft.name} oninput={(event) => onRename(event.currentTarget.value)} />
  </label>

  {#if stale}
    <p class="message warn" role="status">{strings.setlists.changedInReaper} <Button onclick={onReload}>{strings.setlists.reload}</Button></p>
  {/if}

  <div role="list" aria-label={strings.setlists.entriesLabel}>
    {#each draft.entries as entry, at (entry.id)}
      <SetlistEntryRow
        position={at + 1}
        name={nameOf(entry.song_id)}
        missing={missing.includes(entry.id)}
        first={at === 0}
        last={at === draft.entries.length - 1}
        onUp={() => onMove(entry.id, -1)}
        onDown={() => onMove(entry.id, 1)}
        onRemove={() => onRemove(entry.id)}
      />
    {:else}
      <p class="hint">{strings.setlists.empty}</p>
    {/each}
  </div>

  <div class="add">
    <label class="field grow">{strings.setlists.addLabel}
      <select bind:value={chosen}>
        <option value="">{strings.setlists.addPlaceholder}</option>
        {#each songs as song (song.id)}<option value={song.id}>{song.name}</option>{/each}
      </select>
    </label>
    <Button disabled={chosen === ""} onclick={() => { onAdd(chosen); chosen = ""; }}>{strings.setlists.add}</Button>
  </div>

  <div class="actions">
    <Button kind="primary" disabled={!changed || problem !== null} onclick={onSave}>{strings.setlists.save}</Button>
    <Button disabled={!changed} onclick={onDiscard}>{strings.setlists.discard}</Button>
    {#if problem}<span class="message warn" role="status">{strings.setlists.nameNeeded}</span>{:else if changed}<span class="message" role="status">{strings.setlists.unsaved}</span>{/if}
  </div>
</Panel>

<style>
  .field { display: flex; flex-direction: column; gap: 6px; font-size: var(--text-small); font-weight: 600; color: var(--muted); }
  .grow { flex: 1; }
  input, select { height: var(--control-height); border-radius: var(--radius); border: 1px solid var(--line); background: var(--bg); color: var(--text); padding: 0 var(--control-padding); font: inherit; }
  .add { display: flex; align-items: flex-end; gap: var(--space-3); }
  .actions { display: flex; align-items: center; gap: var(--space-3); flex-wrap: wrap; }
  .message { font-size: var(--text-small); color: var(--muted); }
  .warn { color: var(--amber); }
  .hint { color: var(--muted); margin: var(--space-3) 0; }
</style>
