<script lang="ts">
  import { onMount } from "svelte";
  import ComingSoon from "./components/ComingSoon.svelte";
  import Notices from "./components/Notices.svelte";
  import PerformerScreen from "./components/PerformerScreen.svelte";
  import SettingsScreen from "./components/SettingsScreen.svelte";
  import Sidebar from "./components/Sidebar.svelte";
  import { createAppStore } from "./lib/app-store";
  import { connectionBadge } from "./lib/connection";
  import { backend } from "./lib/ipc";
  import { screenLabel, type ScreenId } from "./lib/screens";
  import { fixtureFor } from "./lib/performer-fixtures";
  import { keyIntent, seekCommand, type PerformerPhase, type SeekTarget } from "./lib/performer";
  import { performerView } from "./lib/performer-view";
  import { strings } from "./lib/strings";

  const appState = createAppStore(backend);
  let screen = $state<ScreenId>("player");
  let performerMode = $state(false);

  const phases: PerformerPhase[] = ["Idle", "Playing", "Paused", "CountingIn", "HardStopped"];
  const forced = new URLSearchParams(location.search).get("phase") as PerformerPhase | null;
  const live = $derived($appState.link.live);
  const view = $derived(forced && phases.includes(forced) ? fixtureFor(forced) : performerView($appState.link, $appState.problem));

  const connection = $derived(connectionBadge($appState.link.status, $appState.problem));

  const send = appState.send;

  const playPause = () => send(live?.phase === "Playing" ? "Pause" : "Play");
  const previous = () => send("Previous");
  const rewind = () => send("RestartSong");
  const next = () => send("Next");
  const seek = (target: SeekTarget) => send(seekCommand(target, live?.count_in ?? false));
  const toggleAutoResume = () => send("ToggleAutoResume");

  function onKeydown(event: KeyboardEvent) {
    if (event.repeat || event.metaKey || event.ctrlKey || event.altKey) return;
    const intent = keyIntent(event.key);
    if (!intent) return;
    event.preventDefault();
    if (intent === "PlayPause") playPause();
    else if (intent === "Previous") previous();
    else if (intent === "Next") next();
    else toggleAutoResume();
  }

  onMount(() => appState.start());
</script>

<svelte:window onkeydown={onKeydown} />

<Notices notices={$appState.notices} onDismiss={appState.dismissNotice} />

{#if performerMode}
  <PerformerScreen
    {view}
    onPlayPause={playPause}
    onPrevious={previous}
    onRewind={rewind}
    onNext={next}
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
    onPrevious={previous}
    onRewind={rewind}
    onNext={next}
          onSeek={seek}
          onToggleAutoResume={toggleAutoResume}
          onToggleCountIn={() => send("ToggleCountInOnMarker")}
          onToggleRecord={() => send("ToggleRecordArm")}
        />
        {#if $appState.error}<p class="error">{$appState.error}</p>{/if}
        {#if $appState.pending.length > 0}<p class="pending" role="status">{strings.pending.sending}</p>{/if}
      {:else if screen === "settings"}
        <SettingsScreen />
      {:else}
        <ComingSoon title={screenLabel(screen)} />
      {/if}
    </div>
  </div>
{/if}

<style>
  .layout {
    display: flex;
    height: 100vh;
  }
  .pending {
    color: var(--text-dim, inherit);
    padding: 0 2rem;
  }
  .error {
    color: var(--red);
    padding: 0 2rem;
  }
  .content {
    flex: 1;
    min-width: 0;
    overflow-y: auto;
  }
</style>
