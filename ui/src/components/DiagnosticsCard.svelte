<script lang="ts">
  import { strings } from "../lib/strings";
  import Button from "../kit/Button.svelte";
  import Panel from "../kit/Panel.svelte";
  import { exportDiagnostics } from "../lib/ipc";

  let { level = $bindable(), levels }: { level: string; levels: string[] } = $props();
  let message = $state<{ tone: "ok" | "error"; text: string } | null>(null);

  const exportBundle = async () => {
    try {
      message = { tone: "ok", text: strings.settings.diagnostics.savedTo(await exportDiagnostics()) };
    } catch (e) {
      message = { tone: "error", text: String(e) };
    }
  };
</script>

<Panel title={strings.settings.diagnostics.title} hint={strings.settings.diagnostics.hint}>
  <div class="row">
    <label>{strings.settings.diagnostics.logLevel}
      <select bind:value={level}>
        {#each levels as option (option)}<option value={option}>{option}</option>{/each}
      </select>
    </label>
    <Button onclick={exportBundle}>{strings.settings.diagnostics.export}</Button>
  </div>
  {#if message}<p class="message {message.tone}" role="status">{message.text}</p>{/if}
</Panel>

<style>
  .row { display: flex; gap: 14px; align-items: flex-end; flex-wrap: wrap; }
  label { display: flex; flex-direction: column; gap: 6px; font-size: 13px; font-weight: 600; color: var(--muted); width: 180px; }
  select { height: 44px; border-radius: 10px; border: 1px solid var(--line); background: var(--bg); color: var(--text); padding: 0 14px; font-size: 15px; font-family: inherit; }
  .message { margin: 0; font-size: 14px; word-break: break-all; }
  .message.ok { color: var(--green); }
  .message.error { color: var(--red); }
</style>
