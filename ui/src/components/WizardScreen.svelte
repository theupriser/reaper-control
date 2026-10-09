<script lang="ts">
  import { onMount } from "svelte";
  import Button from "../kit/Button.svelte";
  import WizardSteps from "./WizardSteps.svelte";
  import { currentInstallation, installExtension } from "../lib/ipc";
  import { currentStepIndex, watchInstallation } from "../lib/wizard";
  import { strings } from "../lib/strings";
  import type { InstallationView } from "../lib/generated/protocol";

  let { onclose }: { onclose: () => void } = $props();
  const text = strings.wizard;

  let view = $state<InstallationView | null>(null);
  let error = $state<string | null>(null);
  let busy = $state(false);
  let copied = $state(false);

  const index = $derived(view ? currentStepIndex(view) : 0);
  const step = $derived(view?.steps[index]);

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

<div class="wizard">
  <aside>
    <div class="brand"><span class="logo"><svg viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round"><path d="M3 12h3l2-7 4 14 3-10 2 3h4" /></svg></span><span class="name">{strings.app.name}</span></div>
    <div>
      <h1>{text.title}</h1>
      <p class="intro">{text.intro}</p>
    </div>
    {#if view}<WizardSteps steps={view.steps} />{/if}
  </aside>

  <main>
    {#if view && step}
      <div>
        <div class="count">{text.stepOf(index + 1, view.steps.length)}</div>
        <h2>{text.steps[step.id]}</h2>
        {#if step.advice}<p class="advice">{step.advice}</p>{/if}
      </div>
      {#if view.complete}<p class="done" role="status">{text.done}</p>{/if}
      {#if error}<p class="error" role="alert">{error}</p>{/if}

      <section class="folder">
        <h3>{text.folder}</h3>
        <code>{view.folder}</code>
        <div><Button onclick={copyFolder}>{copied ? text.copied : text.copyFolder}</Button></div>
      </section>

      <div class="spacer"></div>
      <div class="actions">
        {#if view.complete}
          <Button kind="primary" onclick={onclose}>{text.finish}</Button>
        {:else}
          <Button onclick={onclose}>{text.skip}</Button>
          <div class="right">
            <Button onclick={checkAgain}>{text.checkAgain}</Button>
            <Button kind="primary" disabled={!view.can_install || busy} onclick={install}>{busy ? text.installing : text.install}</Button>
          </div>
        {/if}
      </div>
    {:else if error}
      <p class="error" role="alert">{error}</p>
    {/if}
  </main>
</div>

<style>
  .wizard { display: flex; height: 100vh; background: var(--bg); color: var(--text); }
  aside { box-sizing: border-box; display: flex; flex-direction: column; gap: 32px; width: 340px; flex-shrink: 0; padding: 40px 32px; background: var(--sidebar); border-right: 1px solid var(--line); overflow-y: auto; }
  .brand { display: flex; align-items: center; gap: var(--space-3); }
  .logo { display: grid; place-items: center; color: var(--on-accent); width: 40px; height: 40px; border-radius: 11px; background: var(--green); }
  .name { font-size: 18px; font-weight: 800; }
  h1 { margin: 0; font-size: 26px; font-weight: 800; line-height: 1.2; }
  .intro { margin: 10px 0 0; font-size: var(--text-body); line-height: 1.55; color: var(--muted); }
  main { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 22px; padding: 48px 56px; overflow-y: auto; }
  .count { font-size: 13px; font-weight: 700; letter-spacing: 1px; color: var(--amber); }
  h2 { margin: 6px 0 0; font-size: 32px; font-weight: 800; line-height: 1.2; }
  .advice { margin: 8px 0 0; font-size: 16px; line-height: 1.5; color: var(--muted); }
  .folder { display: flex; flex-direction: column; align-items: flex-start; gap: var(--space-3); padding: 22px 24px; background: var(--panel); border: 1px solid var(--line); border-radius: 16px; }
  h3 { margin: 0; font-size: var(--text-title); font-weight: 700; }
  code { word-break: break-all; font-size: 13px; color: var(--text-soft); }
  .spacer { flex: 1; }
  .actions { display: flex; align-items: center; justify-content: space-between; gap: var(--space-3); flex-wrap: wrap; }
  .right { display: flex; gap: var(--space-3); }
  .done { margin: 0; color: var(--green); font-weight: 700; }
  .error { margin: 0; color: var(--red); }
</style>
