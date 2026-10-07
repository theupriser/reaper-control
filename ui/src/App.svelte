<script lang="ts">
  import { onMount } from "svelte";
  import ComingSoon from "./components/ComingSoon.svelte";
  import PerformerScreen from "./components/PerformerScreen.svelte";
  import Sidebar from "./components/Sidebar.svelte";
  import { currentState, dispatch } from "./lib/ipc";
  import { screenLabel, type ScreenId } from "./lib/screens";
  import { fixtureFor } from "./lib/performer-fixtures";
  import type { PerformerPhase } from "./lib/performer";
  import type { AppState, Command } from "./lib/generated/protocol";

  let appState = $state<AppState>({ phase: "Idle" });
  let error = $state<string | null>(null);
  let screen = $state<ScreenId>("player");
  let performerMode = $state(false);

  const phases: PerformerPhase[] = ["Idle", "Playing", "Paused", "CountingIn", "HardStopped"];
  const forced = new URLSearchParams(location.search).get("phase") as PerformerPhase | null;
  const view = $derived(fixtureFor(forced && phases.includes(forced) ? forced : appState.phase));

  const connection = { label: "Fake performance", detail: "no REAPER link yet", tone: "warn" } as const;

  const send = async (command: Command) => {
    try {
      appState = await dispatch(command);
      error = null;
    } catch (e) {
      error = String(e);
    }
  };

  const playPause = () => send(appState.phase === "Playing" ? "Pause" : "Play");

  onMount(async () => {
    try {
      appState = await currentState();
    } catch (e) {
      error = String(e);
    }
  });
</script>

{#if performerMode}
  <PerformerScreen {view} onPlayPause={playPause} onExit={() => (performerMode = false)} />
{:else}
  <div class="layout">
    <Sidebar active={screen} onSelect={(id) => (screen = id)} onPerformer={() => (performerMode = true)} {connection} />
    <div class="content">
      {#if screen === "player"}
        <PerformerScreen {view} onPlayPause={playPause} />
        {#if error}<p class="error">{error}</p>{/if}
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
  .error {
    color: var(--red);
    padding: 0 2rem;
  }
  .content {
    flex: 1;
    min-width: 0;
  }
</style>
