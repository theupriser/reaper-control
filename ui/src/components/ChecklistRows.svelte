<script lang="ts">
  import { strings } from "../lib/strings";
  import type { CheckItem } from "../lib/generated/protocol";

  let { items }: { items: CheckItem[] } = $props();
  const text = strings.checklist;
</script>

<ul>
  {#each items as item (item.id)}
    <li class={item.status}>
      <span class="mark" aria-hidden="true">{item.status === "Passed" ? "✓" : item.status === "Failed" ? "!" : "–"}</span>
      <div>
        <strong>{text.checks[item.id]}</strong>
        <span class="status">{text.status[item.status]}</span>
        <p>{item.detail}</p>
      </div>
    </li>
  {/each}
</ul>

<style>
  ul { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: var(--space-3); }
  li { display: flex; gap: var(--space-3); align-items: flex-start; }
  .mark { width: 32px; height: 32px; border-radius: 50%; display: grid; place-items: center; font-weight: 800; background: var(--raised); border: 1px solid var(--line); flex: none; }
  .Passed .mark { background: var(--green); color: var(--on-accent); border-color: var(--green); }
  .Failed .mark { background: var(--red); color: var(--on-accent); border-color: var(--red); }
  .Skipped { color: var(--muted); }
  strong { font-size: var(--text-body); font-weight: 700; }
  .status { margin-left: var(--space-2); font-size: var(--text-small); color: var(--muted); }
  p { margin: var(--space-1) 0 0 0; font-size: var(--text-small); }
</style>
