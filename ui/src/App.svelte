<script lang="ts">
  import { onMount } from "svelte";
  import ComingSoon from "./components/ComingSoon.svelte";
  import PerformerScreen from "./components/PerformerScreen.svelte";
  import Sidebar from "./components/Sidebar.svelte";
  import { currentState, dispatch } from "./lib/ipc";
  import { screenLabel, type ScreenId } from "./lib/screens";
  import { fixtureFor } from "./lib/performer-fixtures";
  import { keyIntent, seekCommand, type PerformerPhase, type SeekTarget } from "./lib/performer";
  import type { AppState, Command } from "./lib/generated/protocol";

  let appState = $state<AppState>({
    phase: "Idle",
    position: 0,
    auto_resume: true,
    count_in_on_marker: false,
    record_armed: false,
  });
  let error = $state<string | null>(null);
  let screen = $state<ScreenId>("player");
  let performerMode = $state(false);

  const phases: PerformerPhase[] = ["Idle", "Playing", "Paused", "CountingIn", "HardStopped"];
  const forced = new URLSearchParams(location.search).get("phase") as PerformerPhase | null;
  const shown = $derived(fixtureFor(forced && phases.includes(forced) ? forced : appState.phase));
  const view = $derived({
    ...shown,
    songPosition: forced ? shown.songPosition : appState.position,
    totalElapsed: forced ? shown.totalElapsed : appState.position,
    autoResume: appState.auto_resume,
    countInOnMarker: appState.count_in_on_marker,
    recordArmed: appState.record_armed,
  });

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
  const seek = (target: SeekTarget) => send(seekCommand(target, appState.count_in_on_marker));
  const toggleAutoResume = () => send("ToggleAutoResume");

  function onKeydown(event: KeyboardEvent) {
    if (event.repeat || event.metaKey || event.ctrlKey || event.altKey) return;
    const intent = keyIntent(event.key);
    if (!intent) return;
    event.preventDefault();
    if (intent === "PlayPause") playPause();
    else toggleAutoResume();
  }

  onMount(async () => {
    try {
      appState = await currentState();
    } catch (e) {
      error = String(e);
    }
  });
</script>

<svelte:window onkeydown={onKeydown} />

{#if performerMode}
  <PerformerScreen
    {view}
    onPlayPause={playPause}
    onSeek={seek}
    onToggleAutoResume={toggleAutoResume}
    onToggleCountIn={() => send("ToggleCountInOnMarker")}
    onToggleRecord={() => send("ToggleRecordArm")}
    onExit={() => (performerMode = false)}
  />
{:else}
  <div class="layout">
    <Sidebar active={screen} onSelect={(id) => (screen = id)} onPerformer={() => (performerMode = true)} {connection} />
    <div class="content">
      {#if screen === "player"}
        <PerformerScreen
          {view}
          onPlayPause={playPause}
          onSeek={seek}
          onToggleAutoResume={toggleAutoResume}
          onToggleCountIn={() => send("ToggleCountInOnMarker")}
          onToggleRecord={() => send("ToggleRecordArm")}
        />
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
