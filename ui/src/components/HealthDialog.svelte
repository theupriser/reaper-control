<script lang="ts">
  import Button from "../kit/Button.svelte";
  import Dialog from "../kit/Dialog.svelte";
  import { statRows } from "../lib/health";
  import { strings } from "../lib/strings";
  import type { SystemStats } from "../lib/generated/protocol";

  let {
    open,
    connection,
    stats,
    failed,
    onclose,
  }: {
    open: boolean;
    connection: { label: string; detail: string; tone: "ok" | "error" };
    stats: SystemStats | null;
    failed: boolean;
    onclose: () => void;
  } = $props();
</script>

<Dialog {open} title={strings.health.title} {onclose}>
  <p class="state {connection.tone}"><strong>{connection.label}</strong> <span>{connection.detail}</span></p>
  {#if stats}
    <dl>
      {#each statRows(stats) as row (row.label)}<dt>{row.label}</dt><dd>{row.value}</dd>{/each}
    </dl>
  {:else if failed}
    <p role="status">{strings.health.loadFailed}</p>
  {/if}
  {#snippet actions()}<Button onclick={onclose}>{strings.health.close}</Button>{/snippet}
</Dialog>

<style>
  .state { margin: 0 0 var(--space-3) 0; }
  .state.ok strong { color: var(--green); }
  .state.error strong { color: var(--red); }
  span { color: var(--muted); }
  dl { display: grid; grid-template-columns: 1fr auto; gap: var(--space-2) var(--space-5); margin: 0; }
  dt { color: var(--muted); }
  dd { margin: 0; font-weight: 700; font-variant-numeric: tabular-nums; }
</style>
