<script lang="ts">
  import type { Catalog, Command } from "../lib/generated/protocol";
  import * as edit from "../lib/setlist-edit";
  import { strings } from "../lib/strings";
  import DeleteSetlistDialog from "./DeleteSetlistDialog.svelte";
  import LeaveSetlistDialog from "./LeaveSetlistDialog.svelte";
  import SetlistCards from "./SetlistCards.svelte";
  import SetlistEditor from "./SetlistEditor.svelte";

  let { catalog, send, onPlayed }: { catalog: Catalog; send: (command: Command) => Promise<void>; onPlayed: () => void } = $props();

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
  const play = (id: string) => {
    send({ SetActiveSetlist: { id } });
    onPlayed();
  };
  const save = () => draft && send(edit.saveCommand(draft));
  let deletingId = $state<string | null>(null);
  const deleting = $derived(catalog.setlists.find((setlist) => setlist.id === deletingId));
  const remove = () => {
    if (deleting) send(edit.deleteCommand(edit.draftOf(deleting)));
    deletingId = null;
  };
  let leaving = $state(false);
  const back = () => (changed ? (leaving = true) : (draft = null));
  const leave = () => {
    leaving = false;
    draft = null;
  };
  const discard = () => (draft = saved ? edit.draftOf(saved) : null);
</script>

{#if draft}
  <SetlistEditor
    {draft}
    songs={catalog.project_songs}
    {changed}
    problem={edit.saveProblem(draft)}
    {stale}
    saved={saved !== undefined}
    playing={saved !== undefined && catalog.active_setlist === saved.id}
    onBack={back}
    onRename={(name) => change((current) => edit.rename(current, name))}
    onMoveTo={(entryId, index) => change((current) => edit.moveEntryTo(current, entryId, index))}
    onAdd={(songId, index) => change((current) => (index === undefined ? edit.addSong(current, songId) : edit.addSongAt(current, songId, index)))}
    onRemove={(entryId) => change((current) => edit.removeEntry(current, entryId))}
    onSave={save}
    onDiscard={discard}
    onReload={discard}
    onPlay={() => saved && play(saved.id)} />
  <LeaveSetlistDialog open={leaving} onLeave={leave} onStay={() => (leaving = false)} />
{:else}
  <section class="screen">
    <header>
      <div>
        <h1>{strings.setlists.title}</h1>
        <div class="subtitle">{strings.setlists.subtitle}</div>
      </div>
      <span class="saved"><span class="dot"></span>{strings.setlists.savedHint}</span>
    </header>
    <SetlistCards
      setlists={catalog.setlists}
      songs={catalog.project_songs}
      playing={catalog.active_setlist}
      onOpen={open}
      onPlay={play}
      onDelete={(id) => (deletingId = id)}
      onCreate={create} />
    <DeleteSetlistDialog
      open={deleting !== undefined}
      name={deleting?.name ?? ""}
      playing={deleting !== undefined && catalog.active_setlist === deleting.id}
      onConfirm={remove}
      onCancel={() => (deletingId = null)} />
  </section>
{/if}

<style>
  .screen { display: flex; flex-direction: column; gap: 20px; padding: 28px 32px; min-height: 0; }
  header { display: flex; align-items: flex-end; justify-content: space-between; gap: var(--space-4); }
  h1 { margin: 0; font-size: 28px; font-weight: 800; }
  .subtitle { margin-top: 4px; font-size: var(--text-small); color: var(--muted); }
  .saved { display: flex; align-items: center; gap: var(--space-2); font-size: 13px; color: var(--muted); }
  .dot { width: 8px; height: 8px; border-radius: 50%; background: var(--green); }
</style>
