<script lang="ts">
  import { onMount } from "svelte";
  import Button from "../kit/Button.svelte";
  import Panel from "../kit/Panel.svelte";
  import WizardSteps from "./WizardSteps.svelte";
  import { currentInstallation, installExtension } from "../lib/ipc";
  import { watchInstallation } from "../lib/wizard";
  import { strings } from "../lib/strings";
  import type { InstallationView } from "../lib/generated/protocol";

  let { onclose }: { onclose: () => void } = $props();
  const text = strings.wizard;

  let view = $state<InstallationView | null>(null);
  let error = $state<string | null>(null);
  let busy = $state(false);
  let copied = $state(false);

  const install = async () => {
    busy = true;
    error = null;
    try {
      view = await installExtension();
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  };

  const checkAgain = async () => {
    try {
      view = await currentInstallation();
      error = null;
    } catch (e) {
      error = String(e);
    }
  };

  const copyFolder = async () => {
    try {
      await navigator.clipboard.writeText(view?.folder ?? "");
      copied = true;
    } catch {
      copied = false;
    }
  };

  onMount(() => watchInstallation(currentInstallation, (next) => (view = next), 2000));
</script>

<section class="screen">
  <h1>{text.title}</h1>
  <p class="intro">{text.intro}</p>
  {#if view}
    <Panel>
      <WizardSteps steps={view.steps} />
      {#if view.complete}<p class="done" role="status">{text.done}</p>{/if}
      {#if error}<p class="error" role="alert">{error}</p>{/if}
      <div class="actions">
        {#if view.complete}
          <Button kind="primary" onclick={onclose}>{text.finish}</Button>
        {:else}
          <Button kind="primary" disabled={!view.can_install || busy} onclick={install}>{busy ? text.installing : text.install}</Button>
          <Button onclick={checkAgain}>{text.checkAgain}</Button>
          <Button onclick={onclose}>{text.skip}</Button>
        {/if}
      </div>
    </Panel>
    <Panel title={text.folder}>
      <code>{view.folder}</code>
      <div class="actions"><Button onclick={copyFolder}>{copied ? text.copied : text.copyFolder}</Button></div>
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
  .done { color: var(--green); font-weight: 700; margin: 0; }
  .error { color: var(--red); margin: 0; }
  code { word-break: break-all; font-size: var(--text-small); }
</style>
