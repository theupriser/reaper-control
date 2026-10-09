<script lang="ts">
  import ListRow from "../kit/ListRow.svelte";
  import StatusBadge from "../kit/StatusBadge.svelte";
  import { formatTime } from "../lib/performer";
  import type { PlayerRow } from "../lib/player-rows";
  import { strings } from "../lib/strings";

  let { rows }: { rows: PlayerRow[] } = $props();

  const badge = { current: strings.player.current, next: strings.player.next, played: strings.player.played, upcoming: "" };
</script>

<div role="list" aria-label={strings.player.songs}>
  {#each rows as row (row.id)}
    <ListRow selected={row.state === "current"}>
      {#snippet leading()}<span class="number">{row.number}</span>{/snippet}
      <span class:played={row.state === "played"}>{row.name}</span>
      {#snippet trailing()}
        {#if row.hardStop}<StatusBadge tone="error" label={strings.player.hardStop} />{/if}
        {#if row.state === "current"}<StatusBadge tone="ok" label={badge.current} />{/if}
        {#if row.state === "next"}<StatusBadge tone="info" label={badge.next} />{/if}
        {#if row.bpm !== null}<span class="meta">{strings.player.bpm(row.bpm)}</span>{/if}
        <span class="meta time">{formatTime(row.duration)}</span>
      {/snippet}
    </ListRow>
  {/each}
</div>

<style>
  .number { min-width: 2ch; text-align: right; font-variant-numeric: tabular-nums; }
  .played { color: var(--muted); }
  .meta { font-size: var(--text-small); }
  .time { font-variant-numeric: tabular-nums; min-width: 4ch; text-align: right; }
</style>
