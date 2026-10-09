<script lang="ts">
  import { strings } from "../lib/strings";
  import { onMount } from "svelte";
  import { currentTransfer, importSetlists, restoreSetlists } from "../lib/ipc";
  import type { SetlistTransferView } from "../lib/generated/protocol";
  import { doneText, offerText } from "../lib/transfer-text";

  let view = $state<SetlistTransferView | null>(null);
  let chosen = $state<string[]>([]);
  let message = $state<{ tone: "ok" | "error"; text: string } | null>(null);

  const load = async () => {
    try {
      view = await currentTransfer();
      chosen = chosen.filter((id) => view?.imports.some((offer) => offer.id === id));
    } catch (e) {
      message = { tone: "error", text: String(e) };
    }
  };

  const run = async (action: () => Promise<number>, what: "restore" | "import") => {
    try {
      message = { tone: "ok", text: doneText(await action(), what) };
    } catch (e) {
      message = { tone: "error", text: String(e) };
    }
    setTimeout(load, 600);
  };

  onMount(load);
</script>

<div class="card">
  <div class="head">
    <div>
      <h2>{strings.settings.transfer.title}</h2>
      <p class="hint">{strings.settings.transfer.hint}</p>
    </div>
    <button onclick={load}>{strings.settings.transfer.refresh}</button>
  </div>
  {#if view?.problem}<p class="message error">{view.problem}</p>{:else if view}
    <div class="block">
      <h3>{strings.settings.transfer.backup}</h3>
      {#if view.restorable.length === 0}
        <p class="hint">{strings.settings.transfer.nothingToRestore}</p>
      {:else}
        <p>{strings.settings.transfer.missing(view.restorable.join(", "))}</p>
        <div><button class="primary" onclick={() => run(restoreSetlists, "restore")}>{strings.settings.transfer.restore(view.restorable.length)}</button></div>
      {/if}
    </div>
    <div class="block">
      <h3>{strings.settings.transfer.fromV1}</h3>
      {#if view.imports.length === 0}
        <p class="hint">{strings.settings.transfer.nothingToImport}</p>
      {:else}
        {#each view.imports as offer (offer.id)}
          <label class="offer">
            <input type="checkbox" bind:group={chosen} value={offer.id} />
            <span><strong>{offer.name}</strong> <span class="hint">{offerText(offer)}</span></span>
          </label>
        {/each}
        <div><button class="primary" disabled={chosen.length === 0} onclick={() => run(() => importSetlists(chosen), "import")}>{strings.settings.transfer.import(chosen.length)}</button></div>
      {/if}
    </div>
  {/if}
  {#if message}<p class="message {message.tone}" role="status">{message.text}</p>{/if}
</div>

<style>
  .card { background: var(--panel); border: 1px solid var(--line); border-radius: 16px; padding: 20px 24px; display: flex; flex-direction: column; gap: 14px; }
  .head { display: flex; justify-content: space-between; align-items: center; gap: 16px; }
  h2 { margin: 0; font-size: 18px; font-weight: 700; }
  h3 { margin: 0; font-size: 14px; font-weight: 700; color: var(--muted); }
  .block { display: flex; flex-direction: column; gap: 10px; }
  .hint { margin: 0; font-size: 14px; color: var(--muted); }
  .offer { display: flex; align-items: center; gap: 12px; min-height: 44px; font-size: 15px; }
  input[type="checkbox"] { width: 22px; height: 22px; accent-color: var(--green); }
  button { height: var(--control-height); padding: 0 var(--control-padding); border-radius: var(--radius); border: 1px solid var(--line); background: var(--raised); color: var(--text); font-size: 15px; font-weight: 700; cursor: pointer; }
  button.primary { background: var(--green); color: var(--on-accent); border-color: var(--green); }
  button:disabled { opacity: 0.4; cursor: default; }
  .message { margin: 0; font-size: 14px; }
  .message.ok { color: var(--green); }
  .message.error { color: var(--red); }
</style>
