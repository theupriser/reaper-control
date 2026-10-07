<script lang="ts">
  import { onMount } from "svelte";
  import PhaseBadge from "./components/PhaseBadge.svelte";
  import TransportBar from "./components/TransportBar.svelte";
  import { currentState, dispatch } from "./lib/ipc";
  import type { AppState, Command } from "./lib/types";

  let appState = $state<AppState>({ phase: "Idle" });
  let error = $state<string | null>(null);

  const send = async (command: Command) => {
    try {
      appState = await dispatch(command);
      error = null;
    } catch (e) {
      error = String(e);
    }
  };

  onMount(async () => {
    try {
      appState = await currentState();
    } catch (e) {
      error = String(e);
    }
  });
</script>

<main>
  <header>
    <h1>Performer</h1>
    <PhaseBadge phase={appState.phase} />
  </header>
  <p class="hint">App shell stub: commands go through one dispatch to a fake performance.</p>
  <TransportBar onCommand={send} />
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
  .hint {
    color: var(--muted);
    margin: 0;
  }
  .error {
    color: var(--red);
  }
</style>
