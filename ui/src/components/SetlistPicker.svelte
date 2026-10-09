<script lang="ts">
  import { strings } from "../lib/strings";

  let {
    setlists,
    active,
    onChoose,
  }: {
    setlists: { id: string; name: string }[];
    active: string | null;
    onChoose: (id: string | null) => void;
  } = $props();
</script>

<label class="picker">
  <span class="caption">{strings.player.setlist}</span>
  <select aria-label={strings.player.chooseSetlist} value={active ?? ""} onchange={(event) => onChoose(event.currentTarget.value || null)}>
    <option value="">{strings.player.noSetlistChosen}</option>
    {#each setlists as setlist (setlist.id)}<option value={setlist.id}>{setlist.name}</option>{/each}
  </select>
</label>

<style>
  .picker { display: flex; align-items: center; gap: var(--space-3); min-height: var(--control-height); padding: 0 14px; border: 1px solid var(--line); border-radius: var(--radius); background: var(--panel); font-size: var(--text-body); font-weight: 600; }
  .caption { color: var(--muted); font-weight: 500; }
  select { border: none; background: transparent; color: var(--text); font: inherit; cursor: pointer; }
  select option { background: var(--panel); }
</style>
