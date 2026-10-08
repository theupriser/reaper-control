<script lang="ts">
  import { exportDiagnostics } from "../lib/ipc";

  let { level = $bindable(), levels }: { level: string; levels: string[] } = $props();
  let message = $state<{ tone: "ok" | "error"; text: string } | null>(null);

  const exportBundle = async () => {
    try {
      message = { tone: "ok", text: `Saved to ${await exportDiagnostics()}` };
    } catch (e) {
      message = { tone: "error", text: String(e) };
    }
  };
</script>

<div class="card">
  <h2>Diagnostics</h2>
  <p class="hint">The log has the app's lines and the hand-over journal from REAPER. Export one zip to send when something went wrong. The log level applies after a restart.</p>
  <div class="row">
    <label>Log level
      <select bind:value={level}>
        {#each levels as option (option)}<option value={option}>{option}</option>{/each}
      </select>
    </label>
    <button onclick={exportBundle}>Export diagnostics</button>
  </div>
  {#if message}<p class="message {message.tone}" role="status">{message.text}</p>{/if}
</div>

<style>
  .card { background: var(--panel); border: 1px solid var(--line); border-radius: 16px; padding: 20px 24px; display: flex; flex-direction: column; gap: 14px; }
  h2 { margin: 0; font-size: 18px; font-weight: 700; }
  .hint { margin: 0; font-size: 14px; color: var(--muted); }
  .row { display: flex; gap: 14px; align-items: flex-end; flex-wrap: wrap; }
  label { display: flex; flex-direction: column; gap: 6px; font-size: 13px; font-weight: 600; color: var(--muted); width: 180px; }
  select { height: 44px; border-radius: 10px; border: 1px solid var(--line); background: var(--bg); color: var(--text); padding: 0 14px; font-size: 15px; font-family: inherit; }
  button { height: 44px; padding: 0 18px; border-radius: 10px; border: 1px solid var(--line); background: #1e2125; color: var(--text); font-size: 15px; font-weight: 700; cursor: pointer; }
  .message { margin: 0; font-size: 14px; word-break: break-all; }
  .message.ok { color: var(--green); }
  .message.error { color: var(--red); }
</style>
