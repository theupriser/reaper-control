<script lang="ts">
  import Button from "../kit/Button.svelte";
  import { strings } from "../lib/strings";
  import type { CheckId, CheckItem } from "../lib/generated/protocol";

  let { items, onFix }: { items: CheckItem[]; onFix: (id: CheckId) => void } = $props();
  const text = strings.checklist;
  const fixes: Partial<Record<CheckId, string>> = { ExtensionCurrent: text.openSetup, SetlistValid: text.fixInSetlists };
</script>

<ul aria-label={text.checksLabel}>
  {#each items as item (item.id)}
    <li class={item.status}>
      <span class="mark" aria-hidden="true">{item.status === "Passed" ? "✓" : item.status === "Failed" ? "!" : "–"}</span>
      <div class="what">
        <strong>{text.checks[item.id]}</strong>
        <span class="status">{text.status[item.status]}</span>
        <p>{item.detail}</p>
      </div>
      {#if item.status === "Failed" && fixes[item.id]}<Button onclick={() => onFix(item.id)}>{fixes[item.id]}</Button>{/if}
    </li>
  {/each}
</ul>

<style>
  ul { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; }
  li { display: flex; gap: 14px; align-items: center; padding: var(--space-3) 0; border-bottom: 1px solid var(--line-soft); }
  li:last-child { border-bottom: none; }
  .mark { width: 28px; height: 28px; border-radius: 50%; display: grid; place-items: center; font-weight: 900; background: var(--raised); color: var(--muted); flex: none; }
  .Passed .mark { background: var(--tint-ok); color: var(--green); }
  .Failed .mark { background: color-mix(in srgb, var(--amber) 20%, transparent); color: var(--amber); }
  .what { flex: 1; min-width: 0; }
  .Skipped { color: var(--muted); }
  strong { font-size: 16px; font-weight: 700; }
  .status { margin-left: var(--space-2); font-size: var(--text-small); color: var(--muted); }
  p { margin: 2px 0 0; font-size: 13px; color: var(--muted); }
  .Failed p { color: var(--amber); }
</style>
