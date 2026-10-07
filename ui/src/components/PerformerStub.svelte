<script lang="ts">
  import PhaseBadge from "./PhaseBadge.svelte";
  import TransportBar from "./TransportBar.svelte";
  import type { AppState, Command } from "../lib/types";

  let {
    state,
    error,
    onCommand,
    onBack,
  }: {
    state: AppState;
    error: string | null;
    onCommand: (command: Command) => void;
    onBack?: () => void;
  } = $props();
</script>

<main>
  <header>
    <h1>Performer</h1>
    <PhaseBadge phase={state.phase} />
    {#if onBack}<button class="back" onclick={onBack}>Back</button>{/if}
  </header>
  <p class="hint">Stub: commands go through one dispatch to a fake performance.</p>
  <TransportBar {onCommand} />
  {#if error}<p class="error">{error}</p>{/if}
</main>

<style>
  main {
    padding: 28px 32px;
    display: flex;
    flex-direction: column;
    gap: 20px;
  }
  header {
    display: flex;
    align-items: center;
    gap: 16px;
  }
  h1 {
    margin: 0;
    font-size: 28px;
    font-weight: 800;
  }
  .back {
    margin-left: auto;
    min-height: 44px;
    padding: 0 18px;
    border-radius: 10px;
    border: 1px solid var(--line);
    background: transparent;
    color: var(--text);
    font-weight: 700;
    cursor: pointer;
  }
  .hint {
    color: var(--muted);
    margin: 0;
  }
  .error {
    color: var(--red);
  }
</style>
