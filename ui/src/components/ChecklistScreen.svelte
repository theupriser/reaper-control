<script lang="ts">
  import { onMount } from "svelte";
  import Button from "../kit/Button.svelte";
  import ChecklistBanner from "./ChecklistBanner.svelte";
  import ChecklistRows from "./ChecklistRows.svelte";
  import { currentChecklist } from "../lib/ipc";
  import { watchView } from "../lib/watch";
  import { strings } from "../lib/strings";
  import type { CheckId, ChecklistView } from "../lib/generated/protocol";

  let { onSetup, onSetlists, onPerform }: { onSetup: () => void; onSetlists: () => void; onPerform: () => void } = $props();
  const text = strings.checklist;

  let view = $state<ChecklistView | null>(null);
  let error = $state<string | null>(null);

  const checkAgain = async () => {
    try {
      view = await currentChecklist();
      error = null;
    } catch (e) {
      error = String(e);
    }
  };

  const fix = (id: CheckId) => (id === "ExtensionCurrent" ? onSetup() : onSetlists());
  const count = (status: "Passed" | "Failed") => view?.items.filter((item) => item.status === status).length ?? 0;

  onMount(() => watchView(currentChecklist, (next) => (view = next), 3000));
</script>

<section class="screen">
  <header>
    <h1>{text.title}</h1>
    <p class="intro">{text.intro}</p>
  </header>
  {#if view}
    <ChecklistBanner ready={view.ready} passed={count("Passed")} failed={count("Failed")} {onPerform} />
    <section class="card">
      <ChecklistRows items={view.items} onFix={fix} />
    </section>
    <div><Button onclick={checkAgain}>{text.checkAgain}</Button></div>
  {:else if error}
    <p class="error" role="alert">{error}</p>
  {/if}
</section>

<style>
  .screen { padding: 28px 32px; display: flex; flex-direction: column; gap: var(--space-4); }
  h1 { margin: 0; font-size: 28px; font-weight: 800; }
  .intro { margin: 4px 0 0; font-size: 14px; color: var(--muted); }
  .card { padding: 6px 20px; background: var(--panel); border: 1px solid var(--line); border-radius: 16px; }
  .error { margin: 0; color: var(--red); font-weight: 700; }
</style>
