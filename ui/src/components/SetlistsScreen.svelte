<script lang="ts">
  import Button from "../kit/Button.svelte";
  import ListRow from "../kit/ListRow.svelte";
  import StatusBadge from "../kit/StatusBadge.svelte";
  import type { Catalog, Command } from "../lib/generated/protocol";
  import * as edit from "../lib/setlist-edit";
  import { strings } from "../lib/strings";
  import SetlistEditor from "./SetlistEditor.svelte";
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
  <h1>{strings.setlists.title}</h1>
  <div class="columns">
    <div class="side">
      <div role="list" aria-label={strings.setlists.listLabel}>
        {#each catalog.setlists as setlist (setlist.id)}
          <ListRow selected={draft?.id === setlist.id} onclick={() => open(setlist.id)}>
            {setlist.name}
            {#snippet trailing()}
              {#if catalog.active_setlist === setlist.id}<StatusBadge tone="ok" label={strings.setlists.playing} />{/if}
              <span>{strings.setlists.songs(setlist.entries.length)}</span>
            {/snippet}
          </ListRow>
        {:else}
          <p class="hint">{strings.setlists.none}</p>
        {/each}
      </div>
      <Button kind="primary" onclick={create}>{strings.setlists.create}</Button>
    </div>

    {#if draft}
      <div class="editor">
        <SetlistEditor
          {draft}
          songs={catalog.project_songs}
          {changed}
          problem={edit.saveProblem(draft)}
          {stale}
          onRename={(name) => change((current) => edit.rename(current, name))}
          onAdd={(songId) => change((current) => edit.addSong(current, songId))}
          onMove={(entryId, direction) => change((current) => edit.moveEntry(current, entryId, direction))}
          onRemove={(entryId) => change((current) => edit.removeEntry(current, entryId))}
          onSave={save}
          onDiscard={discard}
          onReload={discard}
        />
        {#if saved}
          <Button
            disabled={catalog.active_setlist === saved.id}
            onclick={() => send({ SetActiveSetlist: { id: saved.id } })}>{strings.setlists.play}</Button>
          <Button kind="danger" onclick={() => (deleting = true)}>{strings.setlists.delete}</Button>
          <DeleteSetlistDialog
            open={deleting}
            name={saved.name}
            playing={catalog.active_setlist === saved.id}
            onConfirm={remove}
            onCancel={() => (deleting = false)} />
        {/if}
      </div>
    {/if}
  </div>
</section>

<style>
  .screen { padding: 28px 32px; display: flex; flex-direction: column; gap: var(--space-4); }
  h1 { margin: 0 0 4px 0; font-size: 28px; font-weight: 800; }
  .columns { display: flex; gap: var(--space-5); align-items: flex-start; flex-wrap: wrap; }
  .side { display: flex; flex-direction: column; gap: var(--space-3); width: 320px; }
  .editor { flex: 1; min-width: 360px; display: flex; flex-direction: column; gap: var(--space-3); align-items: flex-start; }
  .editor > :global(*:first-child) { align-self: stretch; }
  .hint { color: var(--muted); margin: 0; }
</style>
