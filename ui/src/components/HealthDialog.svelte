<script lang="ts">
  import Button from "../kit/Button.svelte";
  import Dialog from "../kit/Dialog.svelte";
  import { statRows } from "../lib/health";
  import { usageLevel } from "../lib/performer";
  import { strings } from "../lib/strings";
  import type { SystemStats } from "../lib/generated/protocol";

  let {
    open,
    connection,
    stats,
    failed,
    faulted,
    reenable,
    onReenable,
    onclose,
  }: {
    open: boolean;
    connection: { label: string; detail: string; tone: "ok" | "error" };
    stats: SystemStats | null;
    failed: boolean;
    faulted: boolean;
    reenable: "idle" | "done" | "failed";
    onReenable: () => void;
    onclose: () => void;
  } = $props();
</script>

<Dialog {open} title={strings.health.title} {onclose}>
  <p class="state {connection.tone}"><strong>{connection.label}</strong> <span>{connection.detail}</span></p>
  {#if faulted}
    <div class="reenable">
      {#if reenable === "done"}
        <p role="status">{strings.health.reenabled}</p>
      {:else}
        <Button kind="primary" onclick={onReenable}>{strings.health.reenable}</Button>
        {#if reenable === "failed"}<p role="status">{strings.health.reenableFailed}</p>{/if}
      {/if}
    </div>
  {/if}
  {#if stats}
    <ul class="stats">
      {#each statRows(stats) as row (row.label)}
        <li>
          <span class="name">{row.label}</span>
          <span class="value">{row.value}</span>
          <span class="track" role="presentation">
            {#if row.percent !== null}<span class="bar {usageLevel(row.percent)}" style="width: {Math.max(row.percent, row.percent > 0 ? 1 : 0)}%"></span>{/if}
          </span>
        </li>
      {/each}
    </ul>
  {:else if failed}
    <p role="status">{strings.health.loadFailed}</p>
  {/if}
  {#snippet actions()}<Button onclick={onclose}>{strings.health.close}</Button>{/snippet}
</Dialog>

<style>
  .state { margin: 0 0 var(--space-3) 0; }
  .state.ok strong { color: var(--green); }
  .state.error strong { color: var(--red); }
  .reenable { margin: 0 0 var(--space-3) 0; }
  .reenable p { margin: var(--space-2) 0 0 0; }
  span { color: var(--muted); }
  .stats { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: var(--space-3); min-width: 20rem; }
  li { display: grid; grid-template-columns: 1fr auto; gap: 4px var(--space-5); }
  .name { color: var(--muted); }
  .value { font-weight: 700; font-variant-numeric: tabular-nums; color: var(--text); }
  .track { grid-column: 1 / -1; height: 8px; border-radius: 4px; background: var(--track); overflow: hidden; }
  .bar { display: block; height: 100%; border-radius: 4px; transition: width var(--motion-normal); }
  .low { background: var(--green); }
  .medium { background: var(--amber); }
  .high { background: var(--danger); }
</style>
