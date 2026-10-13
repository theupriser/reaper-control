<script lang="ts">
  import Button from "../kit/Button.svelte";
  import DragGrip from "./DragGrip.svelte";
  import StatusBadge from "../kit/StatusBadge.svelte";
  import type { DragHandlers } from "../lib/drag-drop";
  import { formatTime } from "../lib/performer";
  import type { EntryRow } from "../lib/setlist-rows";
  import { strings } from "../lib/strings";

  let {
    row,
    lifted,
    dropBefore,
    drag,
    onRemove,
  }: {
    row: EntryRow;
    lifted: boolean;
    dropBefore: boolean;
    drag: DragHandlers;
    onRemove: () => void;
  } = $props();
</script>

<div class="row" class:lifted class:gone={row.missing} role="listitem" data-row>
  {#if dropBefore}<span class="line"></span>{/if}
  <DragGrip label={strings.setlists.dragEntry(row.name)} handlers={drag} />
  <span class="position">{row.position}</span>
  <span class="name">{row.name}</span>
  {#if row.missing}<span title={strings.setlists.missingSong}><StatusBadge tone="warn" label={strings.setlists.missing} /></span>{/if}
  <span class="time">{row.duration === null ? strings.setlists.noTime : formatTime(row.duration)}</span>
  <Button kind="danger" label={strings.setlists.remove(row.name)} onclick={onRemove}>✕</Button>
</div>

<style>
  .row { position: relative; box-sizing: border-box; display: flex; align-items: center; gap: 10px; min-height: 50px; padding: 0 6px 0 4px; border-radius: 10px; flex-shrink: 0; }
  .lifted { opacity: 0.45; }
  .line { position: absolute; left: 4px; right: 4px; top: -3px; height: 3px; border-radius: 2px; background: var(--green); }
  .position { min-width: 20px; text-align: right; font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace; font-size: 14px; color: var(--muted); }
  .name { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: 16px; font-weight: 600; }
  .gone { background: color-mix(in srgb, var(--red) 10%, transparent); }
  .gone .name { color: var(--red); }
  .time { min-width: 48px; text-align: right; font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace; font-size: 14px; color: var(--muted); }
</style>
