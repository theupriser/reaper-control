<script lang="ts">
  import type { Catalog, Command } from "../lib/generated/protocol";
  import * as edit from "../lib/setlist-edit";
  import { strings } from "../lib/strings";
  import SetlistEditor from "./SetlistEditor.svelte";
  import SetlistList from "./SetlistList.svelte";
  import SongPool from "./SongPool.svelte";
  import DeleteSetlistDialog from "./DeleteSetlistDialog.svelte";

  let { catalog, send }: { catalog: Catalog; send: (command: Command) => Promise<void> } = $props();

  let draft = $state<edit.SetlistDraft | null>(null);
  const saved = $derived(draft ? catalog.setlists.find((setlist) => setlist.id === draft?.id) : undefined);
  const changed = $derived(draft !== null && edit.isChanged(draft, saved));
  const stale = $derived(draft !== null && saved !== undefined && saved.revision !== draft.revision && changed);

  // Follow the saved setlist while there is nothing of ours to lose (after our own save, or a change in REAPER).
  $effect(() => {
    if (draft && saved && saved.revision !== draft.revision && !changed) draft = edit.draftOf(saved);
  });

  const open = (id: string) => {
    const setlist = catalog.setlists.find((candidate) => candidate.id === id);
    if (setlist) draft = edit.draftOf(setlist);
  };
  const create = () =>
    (draft = edit.emptyDraft(strings.setlists.newName, catalog.setlists.map((setlist) => setlist.id)));
  const change = (next: (current: edit.SetlistDraft) => edit.SetlistDraft) => {
    if (draft) draft = next(draft);
  };
  const save = () => draft && send(edit.saveCommand(draft));
  let deleting = $state(false);
  const remove = () => {
    deleting = false;
    if (!draft) return;
    send(edit.deleteCommand(draft));
    draft = null;
  };
  const discard = () => (draft = saved ? edit.draftOf(saved) : null);
</script>

<section class="screen">
  <header>
    <h1>{strings.setlists.title}</h1>
    <span class="saved"><span class="dot"></span>{strings.setlists.savedHint}</span>
  </header>
  <div class="columns">
    <SetlistList setlists={catalog.setlists} selected={draft?.id ?? null} playing={catalog.active_setlist} onOpen={open} onCreate={create} />

    {#if draft}
      <SetlistEditor
        {draft}
        songs={catalog.project_songs}
        {changed}
        problem={edit.saveProblem(draft)}
        {stale}
        saved={saved !== undefined}
        playing={saved !== undefined && catalog.active_setlist === saved.id}
        onRename={(name) => change((current) => edit.rename(current, name))}
        onMove={(entryId, direction) => change((current) => edit.moveEntry(current, entryId, direction))}
        onRemove={(entryId) => change((current) => edit.removeEntry(current, entryId))}
        onSave={save}
        onDiscard={discard}
        onReload={discard}
        onPlay={() => saved && send({ SetActiveSetlist: { id: saved.id } })}
        onDelete={() => (deleting = true)}
      />
      <SongPool songs={catalog.project_songs} onAdd={(songId) => change((current) => edit.addSong(current, songId))} />
      {#if saved}
        <DeleteSetlistDialog
          open={deleting}
          name={saved.name}
          playing={catalog.active_setlist === saved.id}
          onConfirm={remove}
          onCancel={() => (deleting = false)} />
      {/if}
    {/if}
  </div>
</section>

<style>
  .screen { display: flex; flex-direction: column; gap: var(--space-4); padding: 28px 32px; min-height: 0; }
  header { display: flex; align-items: flex-end; justify-content: space-between; gap: var(--space-4); }
  h1 { margin: 0; font-size: 28px; font-weight: 800; }
  .saved { display: flex; align-items: center; gap: var(--space-2); font-size: 13px; color: var(--muted); }
  .dot { width: 8px; height: 8px; border-radius: 50%; background: var(--green); }
  .columns { display: grid; grid-template-columns: 240px minmax(0, 1fr) 250px; gap: var(--space-4); align-items: start; }
  .columns > :global(:first-child:last-child) { grid-column: 1; }
</style>
