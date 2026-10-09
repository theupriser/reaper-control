<script lang="ts">
  import { strings } from "../lib/strings";
  import type { WizardStep } from "../lib/generated/protocol";

  let { steps }: { steps: WizardStep[] } = $props();
  const text = strings.wizard;
</script>

<ol aria-label={text.stepsLabel}>
  {#each steps as step, index (step.id)}
    <li class={step.status} aria-current={step.status === "Current" || step.status === "NeedsYou" ? "step" : undefined}>
      <span class="number">{step.status === "Done" ? "✓" : index + 1}</span>
      <span class="label">{text.steps[step.id]}</span>
      <span class="status">{text.status[step.status]}</span>
    </li>
  {/each}
</ol>

<style>
  ol { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 6px; }
  li { display: flex; align-items: center; gap: 14px; padding: var(--space-3) 14px; border-radius: 12px; }
  .Current, .NeedsYou { background: var(--raised); }
  .number { display: grid; place-items: center; flex: none; width: 28px; height: 28px; border-radius: 50%; background: var(--line); color: var(--muted); font-size: 14px; font-weight: 800; }
  .Done .number { background: var(--green); color: var(--on-accent); }
  .Current .number { background: var(--amber); color: var(--on-accent); }
  .NeedsYou .number { background: var(--red); color: var(--on-accent); }
  .label { flex: 1; font-size: 16px; font-weight: 700; }
  .Waiting .label { color: var(--muted); }
  .status { font-size: var(--text-small); color: var(--muted); }
</style>
