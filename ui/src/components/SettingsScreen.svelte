<script lang="ts">
  import { onMount } from "svelte";
  import SetlistTransferCard from "./SetlistTransferCard.svelte";
  import { currentSettings, saveSettings } from "../lib/ipc";
  import type { SettingsView } from "../lib/generated/protocol";
  import { ALL_CHANNELS, isChanged, toDraft, toSettings, type SettingsDraft } from "../lib/settings-form";

  let view = $state<SettingsView | null>(null);
  let draft = $state<SettingsDraft | null>(null);
  let message = $state<{ tone: "ok" | "error"; text: string } | null>(null);

  const changed = $derived(view !== null && draft !== null && isChanged(draft, view.settings));
  const devices = $derived(
    view === null || draft === null || draft.device === "" || view.devices.includes(draft.device)
      ? (view?.devices ?? [])
      : [draft.device, ...view.devices],
  );

  const load = async () => {
    try {
      view = await currentSettings();
      draft = toDraft(view.settings);
    } catch (e) {
      message = { tone: "error", text: String(e) };
    }
  };

  const save = async () => {
    if (!draft) return;
    const settings = toSettings(draft);
    if (typeof settings === "string") {
      message = { tone: "error", text: settings };
      return;
    }
    try {
      await saveSettings(settings);
      message = { tone: "ok", text: "Saved. Queue limits apply now; MIDI changes apply after a restart." };
      await load();
    } catch (e) {
      message = { tone: "error", text: String(e) };
    }
  };

  onMount(load);
</script>

<section class="screen">
  <h1>Settings</h1>
  {#if view && draft}
    <div class="card">
      <div class="head">
        <div>
          <h2>MIDI control</h2>
          <p class="hint">Trigger actions from a foot controller or keyboard. Changes apply after a restart.</p>
        </div>
        <label class="toggle"><input type="checkbox" bind:checked={draft.midiEnabled} />Enabled</label>
      </div>
      <div class="row">
        <label class="grow">Device
          <select bind:value={draft.device}>
            <option value="">All devices</option>
            {#each devices as device (device)}<option value={device}>{device}</option>{/each}
          </select>
        </label>
        <label class="channel">Channel
          <select bind:value={draft.channel}>
            <option value={ALL_CHANNELS}>All channels</option>
            {#each Array.from({ length: 16 }, (_, n) => n) as channel (channel)}<option value={String(channel)}>{channel}</option>{/each}
          </select>
        </label>
        <label class="channel">Debounce (ms)<input inputmode="numeric" bind:value={draft.debounce} /></label>
      </div>
      <table>
        <thead><tr><th>Note</th><th>Action</th></tr></thead>
        <tbody>
          {#each view.notes as mapping (mapping.note)}
            <tr><td class="note">{mapping.note}</td><td>{mapping.action}</td></tr>
          {/each}
        </tbody>
      </table>
    </div>

    <div class="card">
      <h2>Command queue</h2>
      <p class="hint">How the app guards the commands it sends to REAPER. Applies at once.</p>
      <div class="row">
        <label class="grow">Repeat window (ms)<input inputmode="numeric" bind:value={draft.repeatWindow} /></label>
        <label class="grow">Timeout (ms)<input inputmode="numeric" bind:value={draft.timeout} /></label>
        <label class="grow">Queue size<input inputmode="numeric" bind:value={draft.capacity} /></label>
      </div>
    </div>

    <SetlistTransferCard />

    <div class="actions">
      <button class="primary" disabled={!changed} onclick={save}>Save</button>
      <button disabled={!changed} onclick={() => view && (draft = toDraft(view.settings))}>Discard changes</button>
      {#if message}<span class="message {message.tone}" role="status">{message.text}</span>{/if}
    </div>
  {:else if message}
    <p class="message error" role="status">{message.text}</p>
  {/if}
</section>

<style>
  .screen { padding: 28px 32px; display: flex; flex-direction: column; gap: 16px; }
  h1 { margin: 0 0 4px 0; font-size: 28px; font-weight: 800; }
  h2 { margin: 0; font-size: 18px; font-weight: 700; }
  .card { background: var(--panel); border: 1px solid var(--line); border-radius: 16px; padding: 20px 24px; display: flex; flex-direction: column; gap: 14px; }
  .head { display: flex; justify-content: space-between; align-items: center; }
  .hint { margin: 4px 0 0 0; font-size: 14px; color: var(--muted); }
  .row { display: flex; gap: 14px; flex-wrap: wrap; }
  .grow { flex: 1; min-width: 160px; }
  .channel { width: 180px; }
  label { display: flex; flex-direction: column; gap: 6px; font-size: 13px; font-weight: 600; color: var(--muted); }
  .toggle { flex-direction: row; align-items: center; gap: 10px; font-size: 14px; color: var(--text); min-height: 44px; }
  input[type="checkbox"] { width: 22px; height: 22px; accent-color: var(--green); }
  select, input:not([type="checkbox"]) { height: 44px; border-radius: 10px; border: 1px solid var(--line); background: var(--bg); color: var(--text); padding: 0 14px; font-size: 15px; font-family: inherit; }
  table { border: 1px solid var(--line); border-radius: 12px; border-collapse: collapse; overflow: hidden; }
  th { text-align: left; padding: 10px 16px; background: #1e2125; font-size: 12px; letter-spacing: 0.6px; color: var(--muted); }
  td { padding: 0 16px; height: 42px; border-top: 1px solid #22262b; font-size: 15px; }
  .note { width: 110px; font-family: ui-monospace, Menlo, monospace; color: var(--amber); font-weight: 700; }
  .actions { display: flex; align-items: center; gap: 12px; flex-wrap: wrap; }
  button { height: 44px; padding: 0 18px; border-radius: 10px; border: 1px solid var(--line); background: #1e2125; color: var(--text); font-size: 15px; font-weight: 700; cursor: pointer; }
  button.primary { background: var(--green); color: #0a1a0b; border-color: var(--green); }
  button:disabled { opacity: 0.4; cursor: default; }
  .message { font-size: 14px; }
  .message.ok { color: var(--green); }
  .message.error { color: var(--red); }
</style>
