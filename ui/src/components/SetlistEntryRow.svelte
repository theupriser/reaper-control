<script lang="ts">
  import Button from "../kit/Button.svelte";
  import ListRow from "../kit/ListRow.svelte";
  import StatusBadge from "../kit/StatusBadge.svelte";
  import { formatTime } from "../lib/performer";
  import type { EntryRow } from "../lib/setlist-rows";
  import { strings } from "../lib/strings";

  let {
    row,
    first,
    last,
    onUp,
    onDown,
    onRemove,
  }: { row: EntryRow; first: boolean; last: boolean; onUp: () => void; onDown: () => void; onRemove: () => void } = $props();
</script>

<ListRow>
  {#snippet leading()}<span class="position">{row.position}</span>{/snippet}
  <span class:gone={row.missing}>{row.name}</span>
  {#snippet trailing()}
    {#if row.missing}<span title={strings.setlists.missingSong}><StatusBadge tone="warn" label={strings.setlists.missing} /></span>{/if}
    <span class="time">{row.duration === null ? strings.setlists.noTime : formatTime(row.duration)}</span>
    <Button label={strings.setlists.moveUp(row.name)} disabled={first} onclick={onUp}>↑</Button>
    <Button label={strings.setlists.moveDown(row.name)} disabled={last} onclick={onDown}>↓</Button>
    <Button kind="danger" label={strings.setlists.remove(row.name)} onclick={onRemove}>✕</Button>
  {/snippet}
</ListRow>

<style>
  .position { min-width: 22px; text-align: right; font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace; font-size: 14px; }
  .gone { color: var(--red); }
  .time { min-width: 50px; text-align: right; font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace; font-size: 14px; }
</style>
