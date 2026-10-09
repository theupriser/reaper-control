<script lang="ts">
  import { strings } from "../lib/strings";
  import type { ActionChoice } from "../lib/generated/protocol";
  import type { NoteRow } from "../lib/settings-form";
  import Button from "../kit/Button.svelte";

  interface Props {
    rows: NoteRow[];
    actions: ActionChoice[];
    onchange: (rows: NoteRow[]) => void;
  }
  let { rows, actions, onchange }: Props = $props();

  const edit = (index: number, part: Partial<NoteRow>) => onchange(rows.map((row, at) => (at === index ? { ...row, ...part } : row)));
  const remove = (index: number) => onchange(rows.filter((_, at) => at !== index));
  const add = () => onchange([...rows, { note: "", action: actions[0]?.id ?? "" }]);
</script>

<div class="table">
  {#if rows.length === 0}
    <p class="empty">{strings.settings.midi.empty}</p>
  {:else}
    <div class="head"><span>{strings.settings.midi.note}</span><span>{strings.settings.midi.action}</span></div>
    {#each rows as row, index (index)}
      <div class="row">
        <input class="note" inputmode="numeric" aria-label={strings.settings.midi.note} value={row.note} oninput={(event) => edit(index, { note: event.currentTarget.value })} />
        <select aria-label={strings.settings.midi.action} value={row.action} onchange={(event) => edit(index, { action: event.currentTarget.value })}>
          {#each actions as action (action.id)}<option value={action.id}>{action.label}</option>{/each}
        </select>
        <Button onclick={() => remove(index)} label={strings.settings.midi.remove(row.note)}>×</Button>
      </div>
    {/each}
  {/if}
  <div><Button onclick={add}>{strings.settings.midi.add}</Button></div>
</div>

<style>
  .table { display: flex; flex-direction: column; gap: 8px; }
  .head, .row { display: grid; grid-template-columns: 110px 1fr 44px; gap: 10px; align-items: center; }
  .head { font-size: 12px; letter-spacing: 0.6px; color: var(--muted); font-weight: 600; padding: 0 2px; }
  .head span:nth-child(2) { grid-column: 2 / 4; }
  .note { font-family: ui-monospace, Menlo, monospace; color: var(--amber); font-weight: 700; }
  select, input { height: 44px; border-radius: 10px; border: 1px solid var(--line); background: var(--bg); color: var(--text); padding: 0 14px; font-size: 15px; font-family: inherit; min-width: 0; }
  .empty { margin: 0; color: var(--muted); font-size: 14px; }
</style>
