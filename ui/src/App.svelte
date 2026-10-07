<script lang="ts">
  import { onMount } from "svelte";
  import ComingSoon from "./components/ComingSoon.svelte";
  import PerformerStub from "./components/PerformerStub.svelte";
  import Sidebar from "./components/Sidebar.svelte";
  import { currentState, dispatch } from "./lib/ipc";
  import { screenLabel, type ScreenId } from "./lib/screens";
  import type { AppState, Command } from "./lib/types";

  let appState = $state<AppState>({ phase: "Idle" });
  let error = $state<string | null>(null);
  let screen = $state<ScreenId>("player");
  let performerMode = $state(false);

  const connection = { label: "Fake performance", detail: "no REAPER link yet", tone: "warn" } as const;

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

{#if performerMode}
  <PerformerStub state={appState} {error} onCommand={send} onBack={() => (performerMode = false)} />
{:else}
  <div class="layout">
    <Sidebar active={screen} onSelect={(id) => (screen = id)} onPerformer={() => (performerMode = true)} {connection} />
    <div class="content">
      {#if screen === "player"}
        <PerformerStub state={appState} {error} onCommand={send} />
      {:else}
        <ComingSoon title={screenLabel(screen)} />
      {/if}
    </div>
  </div>
{/if}

<style>
  .layout {
    display: flex;
  }
  .content {
    flex: 1;
    min-width: 0;
  }
</style>
