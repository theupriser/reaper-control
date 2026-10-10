<script lang="ts">
  import ListRow from "../kit/ListRow.svelte";
  import { formatTime } from "../lib/performer";
  import type { PlayerRow } from "../lib/player-rows";
  import { strings } from "../lib/strings";

  let { rows, onChoose }: { rows: PlayerRow[]; onChoose: (index: number) => void } = $props();
</script>

<div role="group" aria-label={strings.player.songs}>
  {#each rows as row, index (row.id)}
    <ListRow selected={row.state === "current"} onclick={() => onChoose(index)}>
      {#snippet leading()}<span class="number">{row.number}</span>{/snippet}
      <span class="name" class:current={row.state === "current"} class:played={row.state === "played"}>{row.name}</span>
      {#snippet trailing()}
        {#if row.state === "next"}<span class="pill next">{strings.player.next}</span>{/if}
        {#if row.hardStop}<span class="pill stop">{strings.player.hardStopTag}</span>{/if}
        {#if row.bpm !== null}<span class="pill tempo">{strings.player.bpm(row.bpm)}</span>{/if}
        <span class="time">{formatTime(row.duration)}</span>
      {/snippet}
    </ListRow>
  {/each}
</div>

<style>
  .number { min-width: 24px; text-align: right; font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace; font-size: 14px; }
  .name { font-size: 16px; font-weight: 500; color: var(--text-soft); }
  .current { font-weight: 700; color: var(--text); }
  .played { color: var(--muted); }
  .pill { padding: 3px 8px; border-radius: 6px; font-size: 12px; font-weight: 700; }
  .stop { color: var(--red); background: color-mix(in srgb, var(--red) 14%, transparent); }
  .tempo { color: var(--amber); background: color-mix(in srgb, var(--amber) 14%, transparent); }
  .next { color: var(--info); background: color-mix(in srgb, var(--info) 14%, transparent); }
  .time { min-width: 56px; text-align: right; font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace; font-size: 14px; }
</style>
