<script lang="ts">
  import { strings } from "../lib/strings";
  import { onMount } from "svelte";
  import DiagnosticsCard from "./DiagnosticsCard.svelte";
  import SetlistTransferCard from "./SetlistTransferCard.svelte";
  import { currentSettings, saveSettings } from "../lib/ipc";
  import type { SettingsView } from "../lib/generated/protocol";
  import { ALL_CHANNELS, LOG_LEVELS, isChanged, toDraft, toSettings, type SettingsDraft } from "../lib/settings-form";

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
      message = { tone: "ok", text: strings.settings.saved };
      await load();
    } catch (e) {
      message = { tone: "error", text: String(e) };
    }
  };

  onMount(load);
</script>

<section class="screen">
  <h1>{strings.settings.title}</h1>
  {#if view && draft}
    <div class="card">
      <div class="head">
        <div>
          <h2>{strings.settings.midi.title}</h2>
          <p class="hint">{strings.settings.midi.hint}</p>
        </div>
        <label class="toggle"><input type="checkbox" bind:checked={draft.midiEnabled} />{strings.settings.midi.enabled}</label>
      </div>
      <div class="row">
        <label class="grow">{strings.settings.midi.device}
          <select bind:value={draft.device}>
            <option value="">{strings.settings.midi.allDevices}</option>
            {#each devices as device (device)}<option value={device}>{device}</option>{/each}
          </select>
        </label>
        <label class="channel">{strings.settings.midi.channel}
          <select bind:value={draft.channel}>
            <option value={ALL_CHANNELS}>{strings.settings.midi.allChannels}</option>
            {#each Array.from({ length: 16 }, (_, n) => n) as channel (channel)}<option value={String(channel)}>{channel}</option>{/each}
          </select>
        </label>
        <label class="channel">{strings.settings.milliseconds(strings.settings.fields.debounce)}<input inputmode="numeric" bind:value={draft.debounce} /></label>
      </div>
      <table>
        <thead><tr><th>{strings.settings.midi.note}</th><th>{strings.settings.midi.action}</th></tr></thead>
        <tbody>
          {#each view.notes as mapping (mapping.note)}
            <tr><td class="note">{mapping.note}</td><td>{mapping.action}</td></tr>
          {/each}
        </tbody>
      </table>
    </div>

    <div class="card">
      <h2>{strings.settings.queue.title}</h2>
      <p class="hint">{strings.settings.queue.hint}</p>
      <div class="row">
        <label class="grow">{strings.settings.milliseconds(strings.settings.fields.repeatWindow)}<input inputmode="numeric" bind:value={draft.repeatWindow} /></label>
        <label class="grow">{strings.settings.milliseconds(strings.settings.fields.timeout)}<input inputmode="numeric" bind:value={draft.timeout} /></label>
        <label class="grow">{strings.settings.fields.queueSize}<input inputmode="numeric" bind:value={draft.capacity} /></label>
      </div>
    </div>

    <SetlistTransferCard />

    <DiagnosticsCard bind:level={draft.logLevel} levels={LOG_LEVELS} />

    <div class="actions">
      <button class="primary" disabled={!changed} onclick={save}>{strings.settings.save}</button>
      <button disabled={!changed} onclick={() => view && (draft = toDraft(view.settings))}>{strings.settings.discard}</button>
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
  th { text-align: left; padding: 10px 16px; background: var(--raised); font-size: 12px; letter-spacing: 0.6px; color: var(--muted); }
  td { padding: 0 16px; height: 42px; border-top: 1px solid var(--line-soft); font-size: 15px; }
  .note { width: 110px; font-family: ui-monospace, Menlo, monospace; color: var(--amber); font-weight: 700; }
  .actions { display: flex; align-items: center; gap: 12px; flex-wrap: wrap; }
  button { height: var(--control-height); padding: 0 var(--control-padding); border-radius: var(--radius); border: 1px solid var(--line); background: var(--raised); color: var(--text); font-size: 15px; font-weight: 700; cursor: pointer; }
  button.primary { background: var(--green); color: var(--on-accent); border-color: var(--green); }
  button:disabled { opacity: 0.4; cursor: default; }
  .message { font-size: 14px; }
  .message.ok { color: var(--green); }
  .message.error { color: var(--red); }
</style>
