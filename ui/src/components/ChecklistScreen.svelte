<script lang="ts">
  import { onMount } from "svelte";
  import Button from "../kit/Button.svelte";
  import Panel from "../kit/Panel.svelte";
  import ChecklistRows from "./ChecklistRows.svelte";
  import { currentChecklist } from "../lib/ipc";
  import { watchView } from "../lib/watch";
  import { strings } from "../lib/strings";
  import type { ChecklistView } from "../lib/generated/protocol";

  let { onSetup }: { onSetup: () => void } = $props();
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

  const extensionFailed = $derived(view?.items.some((item) => item.id === "ExtensionCurrent" && item.status === "Failed") ?? false);

  onMount(() => watchView(currentChecklist, (next) => (view = next), 3000));
</script>

<section class="screen">
  <h1>{text.title}</h1>
  <p class="intro">{text.intro}</p>
  {#if view}
    <Panel>
      <ChecklistRows items={view.items} />
      <p class={view.ready ? "ready" : "not-ready"} role="status">{view.ready ? text.ready : text.notReady}</p>
      <div class="actions">
        <Button onclick={checkAgain}>{text.checkAgain}</Button>
        {#if extensionFailed}<Button onclick={onSetup}>{text.openSetup}</Button>{/if}
      </div>
    </Panel>
  {:else if error}
    <p class="error" role="alert">{error}</p>
  {/if}
</section>

<style>
  .screen { padding: 28px 32px; display: flex; flex-direction: column; gap: 16px; max-width: 760px; }
  h1 { margin: 0; font-size: 28px; font-weight: 800; }
  .intro { margin: 0; color: var(--muted); }
  .actions { display: flex; gap: 12px; flex-wrap: wrap; }
  .ready { color: var(--green); font-weight: 700; margin: 0; }
  .not-ready, .error { color: var(--red); font-weight: 700; margin: 0; }
</style>
