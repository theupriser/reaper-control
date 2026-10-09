<script lang="ts">
  import { strings } from "../lib/strings";
  import type { WizardStep } from "../lib/generated/protocol";

  let { steps }: { steps: WizardStep[] } = $props();
  const text = strings.wizard;
</script>

<ol>
  {#each steps as step, index (step.id)}
    <li class={step.status} aria-current={step.status === "Current" ? "step" : undefined}>
      <span class="number">{index + 1}</span>
      <div>
        <strong>{text.steps[step.id]}</strong>
        <span class="status">{text.status[step.status]}</span>
        {#if step.advice}<p>{step.advice}</p>{/if}
      </div>
    </li>
  {/each}
</ol>

<style>
  ol { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: var(--space-3); }
  li { display: flex; gap: var(--space-3); align-items: flex-start; }
  .number { width: 32px; height: 32px; border-radius: 50%; display: grid; place-items: center; font-weight: 800; background: var(--raised); border: 1px solid var(--line); flex: none; }
  .Done .number { background: var(--green); color: var(--on-accent); border-color: var(--green); }
  .Current .number { border-color: var(--green); color: var(--green); }
  .NeedsYou .number { background: var(--red); color: var(--on-accent); border-color: var(--red); }
  .Waiting { color: var(--muted); }
  strong { font-size: var(--text-body); font-weight: 700; }
  .status { margin-left: var(--space-2); font-size: var(--text-small); color: var(--muted); }
  p { margin: var(--space-1) 0 0 0; font-size: var(--text-small); }
</style>
